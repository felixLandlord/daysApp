mod generator;

pub use generator::{GenerationOptions, ScheduleGenerator};

use std::collections::{HashMap, HashSet};

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("No employees provided")]
    NoEmployees,
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Schedule generation failed: {0}")]
    GenerationFailed(String),
}

pub type Result<T> = std::result::Result<T, SchedulerError>;
pub type PastSchedules = HashMap<common::EmployeeId, Vec<HashSet<common::Weekday>>>;
