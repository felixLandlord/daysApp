use crate::{Result, StorageError};
use scheduler_core::{AppConfig, Employee, EmployeeId, Schedule, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: PathBuf,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let mut path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("office-scheduler");
        std::fs::create_dir_all(&path).ok();
        path.push("data.db");
        Self { path }
    }
}

pub struct Database {
    db: Surreal<Db>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmployeeRecord {
    id: EmployeeId,
    name: String,
    sex: String,
    role: Option<String>,
    required_days: u8,
    fixed_days: Vec<String>,
    is_mentee: bool,
    is_mentor: bool,
    mentor_id: Option<EmployeeId>,
    works_remote: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScheduleRecord {
    id: String,
    year: i32,
    month: u32,
    assignments: String, // JSON serialized
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigRecord {
    id: String,
    config_data: String, // JSON serialized
}

impl Database {
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let db = Surreal::new::<RocksDb>(config.path)
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;

        db.use_ns("office-scheduler")
            .use_db("main")
            .await
            .map_err(|e| StorageError::ConnectionError(e.to_string()))?;

        Ok(Self { db })
    }

    // Employee operations
    pub async fn save_employee(&self, employee: &Employee) -> Result<()> {
        let record = EmployeeRecord {
            id: employee.id,
            name: employee.name.clone(),
            sex: employee.sex.to_string(),
            role: employee.role.map(|r| r.to_string()),
            required_days: employee.required_days,
            fixed_days: employee.fixed_days.iter().map(|d| d.to_string()).collect(),
            is_mentee: employee.is_mentee,
            is_mentor: employee.is_mentor,
            mentor_id: employee.mentor_id,
            works_remote: employee.works_remote,
        };

        let _: Option<EmployeeRecord> = self
            .db
            .create(("employees", employee.id.to_string()))
            .content(record)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_employee(&self, id: EmployeeId) -> Result<Employee> {
        let record: Option<EmployeeRecord> = self
            .db
            .select(("employees", id.to_string()))
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        record
            .ok_or_else(|| StorageError::NotFound(format!("Employee {}", id)))
            .and_then(|r| self.record_to_employee(r))
    }

    pub async fn get_all_employees(&self) -> Result<Vec<Employee>> {
        let records: Vec<EmployeeRecord> = self
            .db
            .select("employees")
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        records
            .into_iter()
            .map(|r| self.record_to_employee(r))
            .collect()
    }

    pub async fn delete_employee(&self, id: EmployeeId) -> Result<()> {
        let _: Option<EmployeeRecord> = self
            .db
            .delete(("employees", id.to_string()))
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_all_employees(&self) -> Result<()> {
        self.db
            .query("DELETE employees")
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // Schedule operations
    pub async fn save_schedule(&self, schedule: &Schedule) -> Result<()> {
        let id = format!("{}-{}", schedule.year, schedule.month);
        let assignments_json = serde_json::to_string(&schedule.assignments)?;

        let record = ScheduleRecord {
            id: id.clone(),
            year: schedule.year,
            month: schedule.month,
            assignments: assignments_json,
        };

        let _: Option<ScheduleRecord> = self
            .db
            .create(("schedules", id))
            .content(record)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_schedule(&self, year: i32, month: u32) -> Result<Schedule> {
        let id = format!("{}-{}", year, month);
        let record: Option<ScheduleRecord> = self
            .db
            .select(("schedules", id))
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        record
            .ok_or_else(|| StorageError::NotFound(format!("Schedule {}-{}", year, month)))
            .and_then(|r| {
                let assignments: HashMap<Weekday, Vec<EmployeeId>> =
                    serde_json::from_str(&r.assignments)?;
                Ok(Schedule {
                    year: r.year,
                    month: r.month,
                    assignments,
                })
            })
    }

    pub async fn delete_schedule(&self, year: i32, month: u32) -> Result<()> {
        let id = format!("{}-{}", year, month);
        let _: Option<ScheduleRecord> = self
            .db
            .delete(("schedules", id))
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_all_schedules(&self) -> Result<()> {
        self.db
            .query("DELETE schedules")
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_past_schedules(
        &self,
        year: i32,
        month: u32,
        lookback: usize,
    ) -> Result<scheduler_engine::PastSchedules> {
        let mut past_schedules: scheduler_engine::PastSchedules = HashMap::new();

        for i in 1..=lookback {
            let mut past_month = month as i32 - i as i32;
            let mut past_year = year;

            while past_month < 1 {
                past_month += 12;
                past_year -= 1;
            }

            if let Ok(schedule) = self.get_schedule(past_year, past_month as u32).await {
                for (day, employee_ids) in schedule.assignments {
                    for emp_id in employee_ids {
                        past_schedules
                            .entry(emp_id)
                            .or_insert_with(Vec::new)
                            .push(std::iter::once(day).collect());
                    }
                }
            }
        }

        Ok(past_schedules)
    }

    // Config operations
    pub async fn save_config(&self, config: &AppConfig) -> Result<()> {
        let config_json = serde_json::to_string(config)?;
        let record = ConfigRecord {
            id: "app_config".to_string(),
            config_data: config_json,
        };

        let _: Option<ConfigRecord> = self
            .db
            .create(("config", "app_config"))
            .content(record)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_config(&self) -> Result<AppConfig> {
        let record: Option<ConfigRecord> = self
            .db
            .select(("config", "app_config"))
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        match record {
            Some(r) => Ok(serde_json::from_str(&r.config_data)?),
            None => Ok(AppConfig::default()),
        }
    }

    // Helper methods
    fn record_to_employee(&self, record: EmployeeRecord) -> Result<Employee> {
        let sex = match record.sex.as_str() {
            "Male" => scheduler_core::Sex::Male,
            "Female" => scheduler_core::Sex::Female,
            _ => return Err(StorageError::DatabaseError("Invalid sex value".to_string())),
        };

        let role = record.role.and_then(|r| {
            scheduler_core::Role::all()
                .into_iter()
                .find(|role| role.to_string() == r)
        });

        let fixed_days = record
            .fixed_days
            .iter()
            .filter_map(|d| match d.as_str() {
                "Monday" => Some(Weekday::Monday),
                "Tuesday" => Some(Weekday::Tuesday),
                "Wednesday" => Some(Weekday::Wednesday),
                "Thursday" => Some(Weekday::Thursday),
                "Friday" => Some(Weekday::Friday),
                _ => None,
            })
            .collect();

        Ok(Employee {
            id: record.id,
            name: record.name,
            sex,
            role,
            required_days: record.required_days,
            fixed_days,
            is_mentee: record.is_mentee,
            is_mentor: record.is_mentor,
            mentor_id: record.mentor_id,
            works_remote: record.works_remote,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = DatabaseConfig {
            path: temp_dir.path().join("test.db"),
        };
        let db = Database::new(config).await.unwrap();
        (db, temp_dir)
    }

    #[tokio::test]
    async fn test_employee_crud() {
        let (db, _temp) = setup_test_db().await;

        let employee = Employee {
            id: 1,
            name: "Test Employee".to_string(),
            sex: scheduler_core::Sex::Male,
            role: Some(scheduler_core::Role::FullStackEngineer),
            required_days: 2,
            fixed_days: vec![],
            is_mentee: false,
            is_mentor: false,
            mentor_id: None,
            works_remote: false,
        };

        // Create
        db.save_employee(&employee).await.unwrap();

        // Read
        let retrieved = db.get_employee(1).await.unwrap();
        assert_eq!(retrieved.name, "Test Employee");

        // Delete
        db.delete_employee(1).await.unwrap();
        assert!(db.get_employee(1).await.is_err());
    }

    #[tokio::test]
    async fn test_schedule_storage() {
        let (db, _temp) = setup_test_db().await;

        let mut schedule = Schedule::new(2025, 1);
        schedule.assign(Weekday::Monday, 1);
        schedule.assign(Weekday::Monday, 2);

        // Save
        db.save_schedule(&schedule).await.unwrap();

        // Retrieve
        let retrieved = db.get_schedule(2025, 1).await.unwrap();
        assert_eq!(retrieved.count_for_day(Weekday::Monday), 2);
    }
}
