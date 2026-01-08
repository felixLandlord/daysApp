use crate::Result;
use scheduler_core::{Employee, Schedule};
use std::fs;
use std::path::Path;

pub struct JsonExporter;

impl JsonExporter {
    pub fn export_employees<P: AsRef<Path>>(employees: &[Employee], path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(employees)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn export_schedule<P: AsRef<Path>>(schedule: &Schedule, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(schedule)?;
        fs::write(path, json)?;
        Ok(())
    }
}
