use crate::Result;
use scheduler_core::{Employee, Schedule};
use std::fs;
use std::path::Path;

pub struct JsonImporter;

impl JsonImporter {
    pub fn import_employees<P: AsRef<Path>>(path: P) -> Result<Vec<Employee>> {
        let content = fs::read_to_string(path)?;
        let employees: Vec<Employee> = serde_json::from_str(&content)?;
        Ok(employees)
    }

    pub fn import_schedule<P: AsRef<Path>>(path: P) -> Result<Schedule> {
        let content = fs::read_to_string(path)?;
        let schedule: Schedule = serde_json::from_str(&content)?;
        Ok(schedule)
    }
}
