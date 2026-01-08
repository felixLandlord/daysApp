use crate::{IntegrationError, Result};
use scheduler_core::{Employee, EmployeeId, Role, Schedule, Sex, Weekday};
use std::collections::HashMap;
use std::path::Path;

pub struct CsvImporter;

impl CsvImporter {
    pub fn import_employees<P: AsRef<Path>>(path: P) -> Result<Vec<Employee>> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut employees = Vec::new();

        for result in reader.records() {
            let record = result?;

            let id: EmployeeId = record
                .get(0)
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| IntegrationError::ParseError("Invalid ID".to_string()))?;

            let name = record
                .get(1)
                .ok_or_else(|| IntegrationError::ParseError("Missing name".to_string()))?
                .to_string();

            let sex = match record.get(2).unwrap_or("Male") {
                "Female" => Sex::Female,
                _ => Sex::Male,
            };

            let role = record
                .get(3)
                .and_then(|r| Role::all().into_iter().find(|role| role.to_string() == r));

            let required_days: u8 = record.get(4).and_then(|s| s.parse().ok()).unwrap_or(2);

            let fixed_days: Vec<Weekday> = record
                .get(5)
                .map(|s| {
                    s.split(',')
                        .filter_map(|day| match day.trim() {
                            "Monday" => Some(Weekday::Monday),
                            "Tuesday" => Some(Weekday::Tuesday),
                            "Wednesday" => Some(Weekday::Wednesday),
                            "Thursday" => Some(Weekday::Thursday),
                            "Friday" => Some(Weekday::Friday),
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default();

            let is_mentee = record.get(6).and_then(|s| s.parse().ok()).unwrap_or(false);

            let is_mentor = record.get(7).and_then(|s| s.parse().ok()).unwrap_or(false);

            let mentor_id = record.get(8).and_then(|s| s.parse().ok());

            let works_remote = record.get(9).and_then(|s| s.parse().ok()).unwrap_or(false);

            employees.push(Employee {
                id,
                name,
                sex,
                role,
                required_days,
                fixed_days,
                is_mentee,
                is_mentor,
                mentor_id,
                works_remote,
            });
        }

        Ok(employees)
    }

    pub fn import_schedule<P: AsRef<Path>>(path: P) -> Result<Schedule> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut assignments: HashMap<Weekday, Vec<EmployeeId>> = HashMap::new();

        let mut year = 2025;
        let mut month = 1;

        for result in reader.records() {
            let record = result?;

            if record.get(0).unwrap_or("").starts_with("Year") {
                year = record.get(1).and_then(|s| s.parse().ok()).unwrap_or(2025);
                continue;
            }

            if record.get(0).unwrap_or("").starts_with("Month") {
                month = record.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                continue;
            }

            let day_str = record
                .get(0)
                .ok_or_else(|| IntegrationError::ParseError("Missing day column".to_string()))?;

            let day = match day_str {
                "Monday" => Weekday::Monday,
                "Tuesday" => Weekday::Tuesday,
                "Wednesday" => Weekday::Wednesday,
                "Thursday" => Weekday::Thursday,
                "Friday" => Weekday::Friday,
                _ => continue,
            };

            let employee_ids: Vec<EmployeeId> = record
                .iter()
                .skip(1)
                .filter_map(|s| s.parse().ok())
                .collect();

            assignments.insert(day, employee_ids);
        }

        Ok(Schedule {
            year,
            month,
            assignments,
        })
    }
}
