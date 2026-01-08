use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
}

impl Weekday {
    pub fn all() -> Vec<Weekday> {
        vec![
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
        ]
    }
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Weekday::Monday => write!(f, "Monday"),
            Weekday::Tuesday => write!(f, "Tuesday"),
            Weekday::Wednesday => write!(f, "Wednesday"),
            Weekday::Thursday => write!(f, "Thursday"),
            Weekday::Friday => write!(f, "Friday"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub year: i32,
    pub month: u32,
    pub assignments: HashMap<Weekday, Vec<super::EmployeeId>>,
}

impl Schedule {
    pub fn new(year: i32, month: u32) -> Self {
        Self {
            year,
            month,
            assignments: Weekday::all()
                .into_iter()
                .map(|day| (day, Vec::new()))
                .collect(),
        }
    }

    pub fn assign(&mut self, day: Weekday, employee_id: super::EmployeeId) {
        self.assignments.entry(day).or_default().push(employee_id);
    }

    pub fn remove_assignment(&mut self, day: Weekday, employee_id: super::EmployeeId) {
        if let Some(assignments) = self.assignments.get_mut(&day) {
            assignments.retain(|&id| id != employee_id);
        }
    }

    pub fn get_employees_for_day(&self, day: Weekday) -> Vec<super::EmployeeId> {
        self.assignments.get(&day).cloned().unwrap_or_default()
    }

    pub fn count_for_day(&self, day: Weekday) -> usize {
        self.assignments.get(&day).map(|v| v.len()).unwrap_or(0)
    }
}

pub type MonthlySchedule = Schedule;
