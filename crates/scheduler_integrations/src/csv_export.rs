use crate::Result;
use scheduler_core::{Employee, Schedule, Weekday};
use std::path::Path;

pub struct CsvExporter;

impl CsvExporter {
    pub fn export_employees<P: AsRef<Path>>(employees: &[Employee], path: P) -> Result<()> {
        let mut writer = csv::Writer::from_path(path)?;

        writer.write_record(&[
            "ID",
            "Name",
            "Sex",
            "Role",
            "Required Days",
            "Fixed Days",
            "Is Mentee",
            "Is Mentor",
            "Mentor ID",
            "Works Remote",
        ])?;

        for emp in employees {
            writer.write_record(&[
                emp.id.to_string(),
                emp.name.clone(),
                emp.sex.to_string(),
                emp.role.map(|r| r.to_string()).unwrap_or_default(),
                emp.required_days.to_string(),
                emp.fixed_days
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                emp.is_mentee.to_string(),
                emp.is_mentor.to_string(),
                emp.mentor_id.map(|id| id.to_string()).unwrap_or_default(),
                emp.works_remote.to_string(),
            ])?;
        }

        writer.flush()?;
        Ok(())
    }

    pub fn export_schedule<P: AsRef<Path>>(
        schedule: &Schedule,
        employees: &[Employee],
        path: P,
    ) -> Result<()> {
        let mut writer = csv::Writer::from_path(path)?;

        writer.write_record(&["Year", &schedule.year.to_string()])?;
        writer.write_record(&["Month", &schedule.month.to_string()])?;
        writer.write_record(&[""])?;

        let mut header = vec!["Day".to_string()];
        header.extend(employees.iter().map(|e| e.name.clone()));
        writer.write_record(&header)?;

        for day in Weekday::all() {
            let mut row = vec![day.to_string()];
            let day_employees = schedule.get_employees_for_day(day);

            for emp in employees {
                if day_employees.contains(&emp.id) {
                    row.push("X".to_string());
                } else {
                    row.push("".to_string());
                }
            }

            writer.write_record(&row)?;
        }

        writer.flush()?;
        Ok(())
    }
}
