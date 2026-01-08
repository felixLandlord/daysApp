pub mod config;
pub mod employee;
pub mod schedule;

pub use config::{AppConfig, KeyboardShortcuts, MenteeOverlapMode, ScheduleConfig};
pub use employee::{Employee, EmployeeId, Role, Sex};
pub use schedule::{Schedule, Weekday};

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Invalid employee data: {0}")]
    InvalidEmployee(String),
    #[error("Invalid schedule data: {0}")]
    InvalidSchedule(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;
