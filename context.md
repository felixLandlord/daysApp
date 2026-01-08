# Project Structure:
# 
# office-scheduler/
# ├── Cargo.toml
# ├── crates/
# │   ├── core/
# │   │   ├── Cargo.toml
# │   │   └── src/
# │   │       ├── lib.rs
# │   │       ├── employee.rs
# │   │       ├── schedule.rs
# │   │       └── config.rs
# │   ├── scheduler/
# │   │   ├── Cargo.toml
# │   │   └── src/
# │   │       ├── lib.rs
# │   │       ├── generator.rs
# │   │       └── tests.rs
# │   ├── storage/
# │   │   ├── Cargo.toml
# │   │   └── src/
# │   │       ├── lib.rs
# │   │       ├── database.rs
# │   │       └── tests.rs
# │   ├── integrations/
# │   │   ├── Cargo.toml
# │   │   └── src/
# │   │       ├── lib.rs
# │   │       ├── google.rs
# │   │       └── discord.rs
# │   └── ui/
# │       ├── Cargo.toml
# │       └── src/
# │           ├── lib.rs
# │           ├── app.rs
# │           ├── components/
# │           │   ├── mod.rs
# │           │   ├── employees.rs
# │           │   ├── schedules.rs
# │           │   └── settings.rs
# │           └── state.rs
# └── src/
#     └── main.rs

# Root Cargo.toml
[workspace]
members = [
    "crates/core",
    "crates/scheduler",
    "crates/storage",
    "crates/integrations",
    "crates/ui",
]
resolver = "2"

[workspace.package]
version = "0.2.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
rand = "0.8"
tokio = { version = "1.0", features = ["full"] }

[package]
name = "office-scheduler"
version.workspace = true
edition.workspace = true
authors.workspace = true

[dependencies]
core = { path = "crates/core" }
scheduler = { path = "crates/scheduler" }
storage = { path = "crates/storage" }
integrations = { path = "crates/integrations" }
ui = { path = "crates/ui" }
gpui = { git = "https://github.com/zed-industries/zed", rev = "main" }
anyhow.workspace = true
tokio = { workspace = true, features = ["rt-multi-thread"] }

[[bin]]
name = "office-scheduler"
path = "src/main.rs"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.25"
objc = "0.2"

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.52", features = ["Win32_Foundation", "Win32_UI_WindowsAndMessaging"] }


# crates/core/Cargo.toml
[package]
name = "core"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
chrono.workspace = true
thiserror.workspace = true

# crates/core/src/lib.rs
pub mod employee;
pub mod schedule;
pub mod config;

pub use employee::{Employee, Sex, Role, EmployeeId};
pub use schedule::{Schedule, MonthlySchedule, Weekday};
pub use config::{AppConfig, ScheduleConfig, IntegrationConfig, KeyboardShortcuts};

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

# crates/core/src/employee.rs
use serde::{Deserialize, Serialize};
use std::fmt;

pub type EmployeeId = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Employee {
    pub id: EmployeeId,
    pub name: String,
    pub sex: Sex,
    pub role: Option<Role>,
    pub required_days: u8,
    pub fixed_days: Vec<super::Weekday>,
    pub is_mentee: bool,
    pub is_mentor: bool,
    pub mentor_id: Option<EmployeeId>,
    pub works_remote: bool,
}

impl Employee {
    pub fn new(id: EmployeeId, name: String, sex: Sex) -> Self {
        Self {
            id,
            name,
            sex,
            role: None,
            required_days: 2,
            fixed_days: Vec::new(),
            is_mentee: false,
            is_mentor: false,
            mentor_id: None,
            works_remote: false,
        }
    }

    pub fn validate(&self) -> crate::Result<()> {
        if self.name.trim().is_empty() {
            return Err(crate::CoreError::InvalidEmployee("Name cannot be empty".to_string()));
        }
        
        if self.required_days > 5 {
            return Err(crate::CoreError::InvalidEmployee("Required days cannot exceed 5".to_string()));
        }

        if self.is_mentee && self.mentor_id.is_none() {
            return Err(crate::CoreError::InvalidEmployee("Mentee must have a mentor".to_string()));
        }

        if !self.is_mentee && self.mentor_id.is_some() {
            return Err(crate::CoreError::InvalidEmployee("Non-mentee cannot have mentor".to_string()));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sex {
    Male,
    Female,
}

impl fmt::Display for Sex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Sex::Male => write!(f, "Male"),
            Sex::Female => write!(f, "Female"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    HR,
    AiLlmEngineer,
    SocialMediaMarketing,
    ITSupport,
    MLEngineer,
    DataScientist,
    DataAnalyst,
    FullStackEngineer,
    BackendEngineer,
    FrontendEngineer,
    BlockchainEngineer,
    QaEngineer,
    ProjectManager,
    UiUxDesigner,
    MobileEngineer,
    DevOpsEngineer,
    OperationsManager,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Role::HR => write!(f, "Human Resource Manager"),
            Role::AiLlmEngineer => write!(f, "AI-LLM Engineer"),
            Role::SocialMediaMarketing => write!(f, "Social Media Marketing"),
            Role::ITSupport => write!(f, "IT Support"),
            Role::MLEngineer => write!(f, "Machine Learning Engineer"),
            Role::DataScientist => write!(f, "Data Scientist"),
            Role::DataAnalyst => write!(f, "Data Analyst"),
            Role::FullStackEngineer => write!(f, "Full-stack Engineer"),
            Role::BackendEngineer => write!(f, "Backend Engineer"),
            Role::FrontendEngineer => write!(f, "Frontend Engineer"),
            Role::BlockchainEngineer => write!(f, "Blockchain Engineer"),
            Role::QaEngineer => write!(f, "QA Engineer"),
            Role::ProjectManager => write!(f, "Project Manager"),
            Role::UiUxDesigner => write!(f, "UI/UX Designer"),
            Role::MobileEngineer => write!(f, "Mobile Engineer"),
            Role::DevOpsEngineer => write!(f, "DevOps Engineer"),
            Role::OperationsManager => write!(f, "Operations Manager"),
        }
    }
}

impl Role {
    pub fn all() -> Vec<Role> {
        vec![
            Role::HR,
            Role::AiLlmEngineer,
            Role::SocialMediaMarketing,
            Role::ITSupport,
            Role::MLEngineer,
            Role::DataScientist,
            Role::DataAnalyst,
            Role::FullStackEngineer,
            Role::BackendEngineer,
            Role::FrontendEngineer,
            Role::BlockchainEngineer,
            Role::QaEngineer,
            Role::ProjectManager,
            Role::UiUxDesigner,
            Role::MobileEngineer,
            Role::DevOpsEngineer,
            Role::OperationsManager,
        ]
    }
}

# crates/core/src/schedule.rs
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

# crates/core/src/config.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub schedule: ScheduleConfig,
    pub integrations: IntegrationConfig,
    pub shortcuts: KeyboardShortcuts,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schedule: ScheduleConfig::default(),
            integrations: IntegrationConfig::default(),
            shortcuts: KeyboardShortcuts::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub consider_sex_distribution: bool,
    pub consider_role_distribution: bool,
    pub mentee_mentor_overlap: MenteeOverlapMode,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            consider_sex_distribution: false,
            consider_role_distribution: false,
            mentee_mentor_overlap: MenteeOverlapMode::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenteeOverlapMode {
    None,
    AtLeastOne,
    AtLeastTwo,
}

impl std::fmt::Display for MenteeOverlapMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MenteeOverlapMode::None => write!(f, "No overlap required"),
            MenteeOverlapMode::AtLeastOne => write!(f, "At least 1 day"),
            MenteeOverlapMode::AtLeastTwo => write!(f, "At least 2 days"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub google_enabled: bool,
    pub google_sheet_url: Option<String>,
    pub google_share_mode: ShareMode,
    pub discord_enabled: bool,
    pub discord_webhook_url: Option<String>,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            google_enabled: false,
            google_sheet_url: None,
            google_share_mode: ShareMode::ViewOnly,
            discord_enabled: false,
            discord_webhook_url: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShareMode {
    ViewOnly,
    DomainRestricted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardShortcuts {
    pub save: String,
    pub delete: String,
    pub undo: String,
    pub redo: String,
    pub tab_employees: String,
    pub tab_schedules: String,
    pub tab_settings: String,
    pub close_window: String,
    pub minimize: String,
    pub maximize: String,
    pub search: String,
}

impl Default for KeyboardShortcuts {
    fn default() -> Self {
        Self {
            save: "cmd+s".to_string(),
            delete: "cmd+backspace".to_string(),
            undo: "cmd+z".to_string(),
            redo: "cmd+shift+z".to_string(),
            tab_employees: "cmd+1".to_string(),
            tab_schedules: "cmd+2".to_string(),
            tab_settings: "cmd+3".to_string(),
            close_window: "cmd+w".to_string(),
            minimize: "cmd+m".to_string(),
            maximize: "cmd+ctrl+f".to_string(),
            search: "cmd+f".to_string(),
        }
    }
}

# crates/scheduler/Cargo.toml
[package]
name = "scheduler"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
rand.workspace = true
thiserror.workspace = true

[dev-dependencies]
core = { path = "../core" }

# crates/scheduler/src/lib.rs
mod generator;

pub use generator::{ScheduleGenerator, GenerationOptions};

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
pub type PastSchedules = HashMap<core::EmployeeId, Vec<HashSet<core::Weekday>>>;

# crates/scheduler/src/generator.rs
use crate::{PastSchedules, Result, SchedulerError};
use core::{AppConfig, Employee, EmployeeId, Schedule, Sex, Role, Weekday};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, HashSet};

pub struct ScheduleGenerator {
    config: AppConfig,
}

#[derive(Debug, Clone)]
pub struct GenerationOptions {
    pub year: i32,
    pub month: u32,
    pub past_schedules: PastSchedules,
}

impl ScheduleGenerator {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub fn generate(
        &self,
        employees: &[Employee],
        options: GenerationOptions,
    ) -> Result<Schedule> {
        if employees.is_empty() {
            return Err(SchedulerError::NoEmployees);
        }

        let mut schedule = Schedule::new(options.year, options.month);
        let mut day_counts: HashMap<Weekday, usize> = Weekday::all()
            .into_iter()
            .map(|d| (d, 0))
            .collect();

        // Process fixed schedules first
        let flexible_employees = self.process_fixed_schedules(employees, &mut schedule, &mut day_counts)?;

        // Group by required days
        let grouped = self.group_by_required_days(&flexible_employees);

        // Generate combinations
        let combinations = self.generate_day_combinations();

        // Assign flexible employees
        self.assign_flexible_employees(
            &grouped,
            &combinations,
            &mut schedule,
            &mut day_counts,
            &options.past_schedules,
            employees,
        )?;

        // Apply mentee constraints
        self.apply_mentee_constraints(&mut schedule, employees)?;

        Ok(schedule)
    }

    fn process_fixed_schedules(
        &self,
        employees: &[Employee],
        schedule: &mut Schedule,
        day_counts: &mut HashMap<Weekday, usize>,
    ) -> Result<Vec<Employee>> {
        let mut flexible = Vec::new();

        for emp in employees {
            if !emp.fixed_days.is_empty() {
                for day in &emp.fixed_days {
                    schedule.assign(*day, emp.id);
                    *day_counts.entry(*day).or_insert(0) += 1;
                }
            } else {
                flexible.push(emp.clone());
            }
        }

        Ok(flexible)
    }

    fn group_by_required_days(&self, employees: &[Employee]) -> HashMap<usize, Vec<Employee>> {
        let mut grouped: HashMap<usize, Vec<Employee>> = HashMap::new();
        
        for emp in employees {
            grouped
                .entry(emp.required_days as usize)
                .or_default()
                .push(emp.clone());
        }

        let mut rng = thread_rng();
        for group in grouped.values_mut() {
            group.shuffle(&mut rng);
        }

        grouped
    }

    fn generate_day_combinations(&self) -> HashMap<usize, Vec<Vec<Weekday>>> {
        let mut combos = HashMap::new();

        // 1 day
        combos.insert(1, vec![
            vec![Weekday::Monday],
            vec![Weekday::Tuesday],
            vec![Weekday::Wednesday],
            vec![Weekday::Thursday],
            vec![Weekday::Friday],
        ]);

        // 2 days
        combos.insert(2, vec![
            vec![Weekday::Monday, Weekday::Wednesday],
            vec![Weekday::Monday, Weekday::Thursday],
            vec![Weekday::Monday, Weekday::Friday],
            vec![Weekday::Tuesday, Weekday::Thursday],
            vec![Weekday::Tuesday, Weekday::Friday],
            vec![Weekday::Wednesday, Weekday::Friday],
        ]);

        // 3 days
        combos.insert(3, vec![
            vec![Weekday::Monday, Weekday::Wednesday, Weekday::Friday],
        ]);

        // 5 days
        combos.insert(5, vec![
            vec![Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday, Weekday::Thursday, Weekday::Friday],
        ]);

        combos
    }

    fn assign_flexible_employees(
        &self,
        grouped: &HashMap<usize, Vec<Employee>>,
        combinations: &HashMap<usize, Vec<Vec<Weekday>>>,
        schedule: &mut Schedule,
        day_counts: &mut HashMap<Weekday, usize>,
        past_schedules: &PastSchedules,
        all_employees: &[Employee],
    ) -> Result<()> {
        let mut keys: Vec<usize> = grouped.keys().copied().collect();
        keys.sort_by(|a, b| b.cmp(a));

        for num_days in keys {
            if let Some(employees) = grouped.get(&num_days) {
                if let Some(combos) = combinations.get(&num_days) {
                    for emp in employees {
                        let best_combo = self.find_best_combination(
                            combos,
                            day_counts,
                            emp,
                            past_schedules,
                            all_employees,
                        );

                        for day in &best_combo {
                            schedule.assign(*day, emp.id);
                            *day_counts.entry(*day).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn find_best_combination(
        &self,
        combinations: &[Vec<Weekday>],
        day_counts: &HashMap<Weekday, usize>,
        employee: &Employee,
        past_schedules: &PastSchedules,
        all_employees: &[Employee],
    ) -> Vec<Weekday> {
        let mut rng = thread_rng();
        let mut shuffled = combinations.to_vec();
        shuffled.shuffle(&mut rng);

        let mut best_combo = shuffled[0].clone();
        let mut min_score = f64::INFINITY;

        // Calculate past day frequencies
        let past_freqs = self.calculate_past_frequencies(employee, past_schedules);

        for combo in &shuffled {
            let mut temp_counts = day_counts.clone();
            for day in combo {
                *temp_counts.entry(*day).or_insert(0) += 1;
            }

            // Balance score
            let values: Vec<usize> = temp_counts.values().copied().collect();
            let avg = values.iter().sum::<usize>() as f64 / values.len() as f64;
            let variance = values.iter()
                .map(|&v| (v as f64 - avg).powi(2))
                .sum::<f64>();

            // Repetition score
            let repetition = combo.iter()
                .map(|day| past_freqs.get(day).unwrap_or(&0.0))
                .sum::<f64>();

            // Distribution scores
            let sex_score = if self.config.schedule.consider_sex_distribution {
                self.calculate_sex_distribution_score(combo, day_counts, all_employees)
            } else {
                0.0
            };

            let role_score = if self.config.schedule.consider_role_distribution {
                self.calculate_role_distribution_score(combo, day_counts, employee, all_employees)
            } else {
                0.0
            };

            let total = variance + (3.0 * repetition) + sex_score + role_score;

            if total < min_score {
                min_score = total;
                best_combo = combo.clone();
            }
        }

        best_combo
    }

    fn calculate_past_frequencies(
        &self,
        employee: &Employee,
        past_schedules: &PastSchedules,
    ) -> HashMap<Weekday, f64> {
        let mut freqs = HashMap::new();
        
        if let Some(past) = past_schedules.get(&employee.id) {
            let recent = if past.len() > 2 { &past[past.len() - 2..] } else { past };
            
            for (i, schedule) in recent.iter().enumerate() {
                let weight = 1.0 - (i as f64 / recent.len() as f64 * 0.75);
                for day in schedule {
                    *freqs.entry(*day).or_insert(0.0) += weight;
                }
            }
        }

        freqs
    }

    fn calculate_sex_distribution_score(
        &self,
        combo: &[Weekday],
        day_counts: &HashMap<Weekday, usize>,
        all_employees: &[Employee],
    ) -> f64 {
        // Simplified: penalize if adding would make sex distribution very uneven
        let mut score = 0.0;
        
        for day in combo {
            let count = day_counts.get(day).unwrap_or(&0);
            // Penalize if day already has many employees
            if *count > 10 {
                score += 2.0;
            }
        }

        score
    }

    fn calculate_role_distribution_score(
        &self,
        combo: &[Weekday],
        day_counts: &HashMap<Weekday, usize>,
        employee: &Employee,
        all_employees: &[Employee],
    ) -> f64 {
        // Similar to sex distribution
        let mut score = 0.0;
        
        if employee.role.is_none() {
            return score;
        }

        for day in combo {
            let count = day_counts.get(day).unwrap_or(&0);
            if *count > 10 {
                score += 2.0;
            }
        }

        score
    }

    fn apply_mentee_constraints(
        &self,
        schedule: &mut Schedule,
        employees: &[Employee],
    ) -> Result<()> {
        use core::MenteeOverlapMode;

        if matches!(self.config.schedule.mentee_mentor_overlap, MenteeOverlapMode::None) {
            return Ok(());
        }

        for emp in employees {
            if !emp.is_mentee {
                continue;
            }

            let Some(mentor_id) = emp.mentor_id else {
                continue;
            };

            // Find mentor
            let mentor = employees.iter()
                .find(|e| e.id == mentor_id)
                .ok_or_else(|| SchedulerError::GenerationFailed("Mentor not found".to_string()))?;

            // Skip if mentor works fully remote
            if mentor.works_remote {
                continue;
            }

            // Get mentor's days
            let mentor_days: HashSet<Weekday> = Weekday::all()
                .into_iter()
                .filter(|day| {
                    schedule.get_employees_for_day(*day).contains(&mentor_id)
                })
                .collect();

            // Get mentee's days
            let mentee_days: HashSet<Weekday> = Weekday::all()
                .into_iter()
                .filter(|day| {
                    schedule.get_employees_for_day(*day).contains(&emp.id)
                })
                .collect();

            let overlap: Vec<_> = mentee_days.intersection(&mentor_days).collect();

            let required_overlap = match self.config.schedule.mentee_mentor_overlap {
                MenteeOverlapMode::None => 0,
                MenteeOverlapMode::AtLeastOne => 1,
                MenteeOverlapMode::AtLeastTwo => 2,
            };

            if overlap.len() < required_overlap {
                // Adjust schedule to create overlap
                self.adjust_for_mentee_overlap(schedule, emp.id, mentor_id, &mentor_days, required_overlap)?;
            }
        }

        Ok(())
    }

    fn adjust_for_mentee_overlap(
        &self,
        schedule: &mut Schedule,
        mentee_id: EmployeeId,
        mentor_id: EmployeeId,
        mentor_days: &HashSet<Weekday>,
        required: usize,
    ) -> Result<()> {
        // Remove mentee from all days
        for day in Weekday::all() {
            schedule.remove_assignment(day, mentee_id);
        }

        // Reassign to mentor's days (up to required)
        let mut assigned = 0;
        for day in mentor_days.iter().take(required) {
            schedule.assign(*day, mentee_id);
            assigned += 1;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_generation() {
        let config = AppConfig::default();
        let generator = ScheduleGenerator::new(config);

        let employees = vec![
            Employee {
                id: 1,
                name: "Alice".to_string(),
                sex: Sex::Female,
                role: Some(Role::FullStackEngineer),
                required_days: 2,
                fixed_days: vec![],
                is_mentee: false,
                is_mentor: false,
                mentor_id: None,
                works_remote: false,
            },
            Employee {
                id: 2,
                name: "Bob".to_string(),
                sex: Sex::Male,
                role: Some(Role::BackendEngineer),
                required_days: 3,
                fixed_days: vec![],
                is_mentee: false,
                is_mentor: false,
                mentor_id: None,
                works_remote: false,
            },
        ];

        let options = GenerationOptions {
            year: 2025,
            month: 1,
            past_schedules: HashMap::new(),
        };

        let result = generator.generate(&employees, options);
        assert!(result.is_ok());
        
        let schedule = result.unwrap();
        assert_eq!(schedule.year, 2025);
        assert_eq!(schedule.month, 1);
    }

    #[test]
    fn test_mentee_constraint() {
        let mut config = AppConfig::default();
        config.schedule.mentee_mentor_overlap = core::MenteeOverlapMode::AtLeastOne;
        
        let generator = ScheduleGenerator::new(config);

        let employees = vec![
            Employee {
                id: 1,
                name: "Mentor".to_string(),
                sex: Sex::Male,
                role: None,
                required_days: 2,
                fixed_days: vec![Weekday::Monday, Weekday::Wednesday],
                is_mentee: false,
                is_mentor: true,
                mentor_id: None,
                works_remote: false,
            },
            Employee {
                id: 2,
                name: "Mentee".to_string(),
                sex: Sex::Female,
                role: None,
                required_days: 2,
                fixed_days: vec![],
                is_mentee: true,
                is_mentor: false,
                mentor_id: Some(1),
                works_remote: false,
            },
        ];

        let options = GenerationOptions {
            year: 2025,
            month: 1,
            past_schedules: HashMap::new(),
        };

        let result = generator.generate(&employees, options);
        assert!(result.is_ok());
    }
}

# crates/storage/Cargo.toml
[package]
name = "storage"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
surrealdb = { version = "2.0", features = ["kv-rocksdb"] }
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
chrono.workspace = true

[dev-dependencies]
tempfile = "3.8"

# crates/storage/src/lib.rs
mod database;

pub use database::{Database, DatabaseConfig};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

# crates/storage/src/database.rs
use crate::{Result, StorageError};
use core::{AppConfig, Employee, EmployeeId, Schedule, Weekday};
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
    ) -> Result<scheduler::PastSchedules> {
        let mut past_schedules: scheduler::PastSchedules = HashMap::new();

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
            "Male" => core::Sex::Male,
            "Female" => core::Sex::Female,
            _ => return Err(StorageError::DatabaseError("Invalid sex value".to_string())),
        };

        let role = record.role.and_then(|r| {
            core::Role::all().into_iter().find(|role| role.to_string() == r)
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
            sex: core::Sex::Male,
            role: Some(core::Role::FullStackEngineer),
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

# crates/integrations/Cargo.toml
[package]
name = "integrations"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
reqwest = { version = "0.11", features = ["json"] }

# crates/integrations/src/lib.rs
pub mod google;
pub mod discord;

#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Integration not configured")]
    NotConfigured,
    #[error("Integration error: {0}")]
    GeneralError(String),
}

pub type Result<T> = std::result::Result<T, IntegrationError>;

# crates/integrations/src/google.rs
use crate::{IntegrationError, Result};
use core::{Employee, Schedule, Weekday};
use serde_json::json;

pub struct GoogleSheetsClient {
    api_key: Option<String>,
    client: reqwest::Client,
}

impl GoogleSheetsClient {
    pub fn new() -> Self {
        Self {
            api_key: None,
            client: reqwest::Client::new(),
        }
    }

    pub fn configure(&mut self, api_key: String) {
        self.api_key = Some(api_key);
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    // Placeholder for importing employees from Google Sheets
    pub async fn import_employees(&self, sheet_url: &str) -> Result<Vec<Employee>> {
        if !self.is_configured() {
            return Err(IntegrationError::NotConfigured);
        }

        // TODO: Implement actual Google Sheets API integration
        // This would involve:
        // 1. Parsing the sheet URL to extract sheet ID
        // 2. Making authenticated requests to Google Sheets API
        // 3. Parsing the response and converting to Employee structs
        
        // For now, return empty vector as foundation
        Ok(Vec::new())
    }

    // Placeholder for importing schedule from Google Sheets
    pub async fn import_schedule(&self, sheet_url: &str) -> Result<Schedule> {
        if !self.is_configured() {
            return Err(IntegrationError::NotConfigured);
        }

        // TODO: Implement actual Google Sheets API integration
        
        // For now, return empty schedule
        Ok(Schedule::new(2025, 1))
    }

    // Placeholder for exporting schedule to Google Sheets
    pub async fn export_schedule(
        &self,
        schedule: &Schedule,
        employees: &[Employee],
        share_mode: core::ShareMode,
    ) -> Result<String> {
        if !self.is_configured() {
            return Err(IntegrationError::NotConfigured);
        }

        // TODO: Implement actual Google Sheets API integration
        // This would involve:
        // 1. Creating a new Google Sheet
        // 2. Formatting the schedule data
        // 3. Writing to the sheet
        // 4. Setting appropriate sharing permissions based on share_mode
        // 5. Returning the shareable link
        
        // For now, return placeholder URL
        Ok("https://docs.google.com/spreadsheets/d/placeholder".to_string())
    }

    // Helper to convert schedule to spreadsheet format
    fn schedule_to_sheet_data(
        &self,
        schedule: &Schedule,
        employees: &[Employee],
    ) -> Vec<Vec<String>> {
        let mut data = Vec::new();
        
        // Header row
        let mut header = vec!["Name".to_string()];
        for day in Weekday::all() {
            header.push(day.to_string());
        }
        data.push(header);

        // Count row
        let mut counts = vec!["".to_string()];
        for day in Weekday::all() {
            counts.push(schedule.count_for_day(day).to_string());
        }
        data.push(counts);

        // Employee rows
        for emp in employees {
            let mut row = vec![emp.name.clone()];
            for day in Weekday::all() {
                let assigned = schedule
                    .get_employees_for_day(day)
                    .contains(&emp.id);
                row.push(if assigned { "X".to_string() } else { "".to_string() });
            }
            data.push(row);
        }

        data
    }
}

impl Default for GoogleSheetsClient {
    fn default() -> Self {
        Self::new()
    }
}

# crates/integrations/src/discord.rs
use crate::{IntegrationError, Result};
use core::Schedule;
use serde_json::json;

pub struct DiscordClient {
    webhook_url: Option<String>,
    client: reqwest::Client,
}

impl DiscordClient {
    pub fn new() -> Self {
        Self {
            webhook_url: None,
            client: reqwest::Client::new(),
        }
    }

    pub fn configure(&mut self, webhook_url: String) {
        self.webhook_url = Some(webhook_url);
    }

    pub fn is_configured(&self) -> bool {
        self.webhook_url.is_some()
    }

    // Placeholder for sending schedule notification to Discord
    pub async fn send_schedule_notification(
        &self,
        schedule: &Schedule,
        message: Option<String>,
    ) -> Result<()> {
        let Some(webhook_url) = &self.webhook_url else {
            return Err(IntegrationError::NotConfigured);
        };

        // TODO: Implement actual Discord webhook integration
        // This would involve:
        // 1. Formatting the schedule into a Discord-friendly message
        // 2. Creating embeds for better visualization
        // 3. Sending the webhook request
        
        let payload = json!({
            "content": message.unwrap_or_else(|| "Schedule updated".to_string()),
            "embeds": [{
                "title": format!("Schedule for {}/{}", schedule.month, schedule.year),
                "description": "New office schedule has been generated",
                "color": 5814783,
            }]
        });

        // Placeholder for actual HTTP request
        // self.client.post(webhook_url)
        //     .json(&payload)
        //     .send()
        //     .await?;

        Ok(())
    }

    // Helper to format schedule summary
    fn format_schedule_summary(&self, schedule: &Schedule) -> String {
        let mut summary = format!("**Schedule for {}/{}**\n\n", schedule.month, schedule.year);
        
        for day in core::Weekday::all() {
            let count = schedule.count_for_day(day);
            summary.push_str(&format!("**{}**: {} employees\n", day, count));
        }

        summary
    }
}

impl Default for DiscordClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_client_configuration() {
        let mut client = GoogleSheetsClient::new();
        assert!(!client.is_configured());
        
        client.configure("test_api_key".to_string());
        assert!(client.is_configured());
    }

    #[test]
    fn test_discord_client_configuration() {
        let mut client = DiscordClient::new();
        assert!(!client.is_configured());
        
        client.configure("https://discord.com/api/webhooks/test".to_string());
        assert!(client.is_configured());
    }
}

# crates/ui/Cargo.toml
[package]
name = "ui"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
scheduler = { path = "../scheduler" }
storage = { path = "../storage" }
integrations = { path = "../integrations" }
gpui = { git = "https://github.com/zed-industries/zed", rev = "main" }
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
chrono.workspace = true

# crates/ui/src/lib.rs
pub mod app;
pub mod components;
pub mod state;

pub use app::App;
pub use state::AppState;

# crates/ui/src/state.rs
use core::{AppConfig, Employee, EmployeeId, Schedule};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<RwLock<AppStateInner>>,
}

struct AppStateInner {
    pub employees: Vec<Employee>,
    pub current_schedule: Option<Schedule>,
    pub config: AppConfig,
    pub current_tab: Tab,
    pub search_query: String,
    pub is_editing: bool,
    pub undo_stack: Vec<UndoAction>,
    pub redo_stack: Vec<UndoAction>,
    pub selected_year: i32,
    pub selected_month: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Employees,
    Schedules,
    Settings,
}

#[derive(Debug, Clone)]
pub enum UndoAction {
    AddEmployee(Employee),
    DeleteEmployee(EmployeeId),
    UpdateEmployee { old: Employee, new: Employee },
    UpdateSchedule { old: Schedule, new: Schedule },
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let now = chrono::Local::now();
        Self {
            inner: Arc::new(RwLock::new(AppStateInner {
                employees: Vec::new(),
                current_schedule: None,
                config,
                current_tab: Tab::Schedules,
                search_query: String::new(),
                is_editing: false,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
                selected_year: now.year(),
                selected_month: now.month(),
            })),
        }
    }

    pub async fn get_employees(&self) -> Vec<Employee> {
        self.inner.read().await.employees.clone()
    }

    pub async fn set_employees(&self, employees: Vec<Employee>) {
        self.inner.write().await.employees = employees;
    }

    pub async fn add_employee(&self, employee: Employee) {
        let mut inner = self.inner.write().await;
        inner.undo_stack.push(UndoAction::AddEmployee(employee.clone()));
        inner.redo_stack.clear();
        inner.employees.push(employee);
    }

    pub async fn update_employee(&self, employee: Employee) {
        let mut inner = self.inner.write().await;
        if let Some(pos) = inner.employees.iter().position(|e| e.id == employee.id) {
            let old = inner.employees[pos].clone();
            inner.undo_stack.push(UndoAction::UpdateEmployee {
                old: old.clone(),
                new: employee.clone(),
            });
            inner.redo_stack.clear();
            inner.employees[pos] = employee;
        }
    }

    pub async fn delete_employee(&self, id: EmployeeId) {
        let mut inner = self.inner.write().await;
        if let Some(pos) = inner.employees.iter().position(|e| e.id == id) {
            let employee = inner.employees.remove(pos);
            inner.undo_stack.push(UndoAction::DeleteEmployee(id));
            inner.redo_stack.clear();
        }
    }

    pub async fn get_schedule(&self) -> Option<Schedule> {
        self.inner.read().await.current_schedule.clone()
    }

    pub async fn set_schedule(&self, schedule: Option<Schedule>) {
        let mut inner = self.inner.write().await;
        if let Some(old) = inner.current_schedule.clone() {
            if let Some(new) = schedule.clone() {
                inner.undo_stack.push(UndoAction::UpdateSchedule { old, new: new.clone() });
                inner.redo_stack.clear();
            }
        }
        inner.current_schedule = schedule;
    }

    pub async fn get_config(&self) -> AppConfig {
        self.inner.read().await.config.clone()
    }

    pub async fn set_config(&self, config: AppConfig) {
        self.inner.write().await.config = config;
    }

    pub async fn get_tab(&self) -> Tab {
        self.inner.read().await.current_tab
    }

    pub async fn set_tab(&self, tab: Tab) {
        self.inner.write().await.current_tab = tab;
    }

    pub async fn get_search_query(&self) -> String {
        self.inner.read().await.search_query.clone()
    }

    pub async fn set_search_query(&self, query: String) {
        self.inner.write().await.search_query = query;
    }

    pub async fn is_editing(&self) -> bool {
        self.inner.read().await.is_editing
    }

    pub async fn set_editing(&self, editing: bool) {
        self.inner.write().await.is_editing = editing;
    }

    pub async fn undo(&self) {
        let mut inner = self.inner.write().await;
        if let Some(action) = inner.undo_stack.pop() {
            match action.clone() {
                UndoAction::AddEmployee(emp) => {
                    inner.employees.retain(|e| e.id != emp.id);
                }
                UndoAction::DeleteEmployee(id) => {
                    // Can't restore without saved data
                }
                UndoAction::UpdateEmployee { old, new } => {
                    if let Some(pos) = inner.employees.iter().position(|e| e.id == new.id) {
                        inner.employees[pos] = old;
                    }
                }
                UndoAction::UpdateSchedule { old, new } => {
                    inner.current_schedule = Some(old);
                }
            }
            inner.redo_stack.push(action);
        }
    }

    pub async fn redo(&self) {
        let mut inner = self.inner.write().await;
        if let Some(action) = inner.redo_stack.pop() {
            match action.clone() {
                UndoAction::AddEmployee(emp) => {
                    inner.employees.push(emp);
                }
                UndoAction::DeleteEmployee(id) => {
                    inner.employees.retain(|e| e.id != id);
                }
                UndoAction::UpdateEmployee { old, new } => {
                    if let Some(pos) = inner.employees.iter().position(|e| e.id == old.id) {
                        inner.employees[pos] = new;
                    }
                }
                UndoAction::UpdateSchedule { old, new } => {
                    inner.current_schedule = Some(new);
                }
            }
            inner.undo_stack.push(action);
        }
    }

    pub async fn get_selected_date(&self) -> (i32, u32) {
        let inner = self.inner.read().await;
        (inner.selected_year, inner.selected_month)
    }

    pub async fn set_selected_date(&self, year: i32, month: u32) {
        let mut inner = self.inner.write().await;
        inner.selected_year = year;
        inner.selected_month = month;
    }

    pub async fn get_filtered_employees(&self) -> Vec<Employee> {
        let inner = self.inner.read().await;
        let query = inner.search_query.to_lowercase();
        
        if query.is_empty() {
            inner.employees.clone()
        } else {
            inner
                .employees
                .iter()
                .filter(|e| {
                    e.name.to_lowercase().contains(&query)
                        || e.role.map(|r| r.to_string().to_lowercase().contains(&query)).unwrap_or(false)
                })
                .cloned()
                .collect()
        }
    }

    pub async fn next_employee_id(&self) -> EmployeeId {
        self.inner
            .read()
            .await
            .employees
            .iter()
            .map(|e| e.id)
            .max()
            .unwrap_or(0)
            + 1
    }
}

# src/main.rs
use anyhow::Result;
use gpui::*;
use ui::App;

fn main() -> Result<()> {
    env_logger::init();
    
    App::new().run(|cx: &mut AppContext| {
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::new(Pixels(100.0), Pixels(100.0)),
                    size: Size {
                        width: Pixels(1200.0),
                        height: Pixels(800.0),
                    },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("Office Scheduler".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                window_min_size: Some(Size {
                    width: Pixels(800.0),
                    height: Pixels(600.0),
                }),
                ..Default::default()
            },
            |cx| {
                cx.new_view(|cx| ui::App::new(cx))
            },
        );
    })
}

# crates/ui/src/app.rs
use crate::components::{EmployeesView, SchedulesView, SettingsView};
use crate::state::{AppState, Tab};
use core::AppConfig;
use gpui::*;
use storage::{Database, DatabaseConfig};
use std::sync::Arc;

pub struct App {
    state: AppState,
    database: Arc<Database>,
}

impl App {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        // Initialize database
        let db_config = DatabaseConfig::default();
        let database = cx.background_executor().block(
            Database::new(db_config)
        ).expect("Failed to initialize database");
        
        let database = Arc::new(database);

        // Load config
        let config = cx.background_executor().block(database.get_config())
            .unwrap_or_default();

        // Load employees
        let employees = cx.background_executor().block(database.get_all_employees())
            .unwrap_or_default();

        let state = AppState::new(config);
        cx.background_executor().spawn({
            let state = state.clone();
            async move {
                state.set_employees(employees).await;
            }
        }).detach();

        Self { state, database }
    }

    pub fn save_employee(&mut self, employee: core::Employee, cx: &mut ViewContext<Self>) {
        let db = self.database.clone();
        let state = self.state.clone();
        
        cx.spawn(|_, _| async move {
            if let Ok(_) = db.save_employee(&employee).await {
                state.add_employee(employee).await;
            }
        }).detach();
    }

    pub fn generate_schedule(&mut self, cx: &mut ViewContext<Self>) {
        let db = self.database.clone();
        let state = self.state.clone();
        
        cx.spawn(|_, _| async move {
            let employees = state.get_employees().await;
            let (year, month) = state.get_selected_date().await;
            let config = state.get_config().await;
            
            // Get past schedules
            let past = db.get_past_schedules(year, month, 3).await.unwrap_or_default();
            
            let generator = scheduler::ScheduleGenerator::new(config);
            let options = scheduler::GenerationOptions {
                year,
                month,
                past_schedules: past,
            };
            
            if let Ok(schedule) = generator.generate(&employees, options) {
                state.set_schedule(Some(schedule)).await;
            }
        }).detach();
    }

    pub fn save_schedule(&mut self, cx: &mut ViewContext<Self>) {
        let db = self.database.clone();
        let state = self.state.clone();
        
        cx.spawn(|_, _| async move {
            if let Some(schedule) = state.get_schedule().await {
                let _ = db.save_schedule(&schedule).await;
            }
        }).detach();
    }
}

impl Render for App {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let tab = cx.background_executor().block(self.state.get_tab());
        
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0xffffff))
            .child(self.render_menu_bar(cx))
            .child(self.render_tab_bar(cx))
            .child(self.render_content(tab, cx))
    }
}

impl App {
    fn render_menu_bar(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let tab = cx.background_executor().block(self.state.get_tab());
        let is_editing = cx.background_executor().block(self.state.is_editing());
        
        div()
            .flex()
            .items_center()
            .justify_between()
            .h_12()
            .px_4()
            .border_b_1()
            .border_color(rgb(0xe0e0e0))
            .child(
                // Left section - Tab info
                div()
                    .flex()
                    .gap_4()
                    .when(tab == Tab::Employees, |this| {
                        let count = cx.background_executor().block({
                            let state = self.state.clone();
                            async move { state.get_employees().await.len() }
                        });
                        this.child(format!("Total: {}", count))
                    })
                    .when(tab == Tab::Schedules, |this| {
                        // Show day counts
                        this.child("Schedule Info")
                    })
            )
            .child(
                // Right section - Actions
                div()
                    .flex()
                    .gap_2()
                    .when(tab == Tab::Employees, |this| {
                        this.child(Button::new("edit").label("Edit")
                            .on_click(cx.listener(|this, _, cx| {
                                let state = this.state.clone();
                                cx.spawn(|_, _| async move {
                                    let current = state.is_editing().await;
                                    state.set_editing(!current).await;
                                }).detach();
                                cx.notify();
                            })))
                           .child(Button::new("save").label("Save"))
                           .child(Button::new("import").label("Import"))
                    })
                    .when(tab == Tab::Schedules, |this| {
                        this.child(Button::new("generate").label("Generate")
                            .on_click(cx.listener(|this, _, cx| {
                                this.generate_schedule(cx);
                                cx.notify();
                            })))
                           .child(Button::new("save").label("Save")
                            .on_click(cx.listener(|this, _, cx| {
                                this.save_schedule(cx);
                                cx.notify();
                            })))
                           .child(Button::new("export").label("Export"))
                    })
                    .child(
                        // Search bar
                        div()
                            .child(TextInput::new("search"))
                    )
            )
    }

    fn render_tab_bar(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let current_tab = cx.background_executor().block(self.state.get_tab());
        
        div()
            .flex()
            .h_12()
            .border_b_1()
            .border_color(rgb(0xe0e0e0))
            .child(self.render_tab(Tab::Employees, "Employees", current_tab, cx))
            .child(self.render_tab(Tab::Schedules, "Schedules", current_tab, cx))
            .child(self.render_tab(Tab::Settings, "Settings", current_tab, cx))
    }

    fn render_tab(&mut self, tab: Tab, label: &str, current: Tab, cx: &mut ViewContext<Self>) -> impl IntoElement {
        Button::new(format!("tab-{:?}", tab))
            .label(label)
            .selected(tab == current)
            .on_click(cx.listener(move |this, _, cx| {
                let state = this.state.clone();
                cx.spawn(|_, _| async move {
                    state.set_tab(tab).await;
                }).detach();
                cx.notify();
            }))
    }

    fn render_content(&mut self, tab: Tab, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .overflow_y_scroll()
            .p_4()
            .child(match tab {
                Tab::Employees => EmployeesView::new(self.state.clone()).into_any_element(),
                Tab::Schedules => SchedulesView::new(self.state.clone()).into_any_element(),
                Tab::Settings => SettingsView::new(self.state.clone()).into_any_element(),
            })
    }
}

# crates/ui/src/components/mod.rs
pub mod employees;
pub mod schedules;
pub mod settings;

pub use employees::EmployeesView;
pub use schedules::SchedulesView;
pub use settings::SettingsView;

# crates/ui/src/components/employees.rs
use crate::state::AppState;
use core::Employee;
use gpui::*;

pub struct EmployeesView {
    state: AppState,
}

impl EmployeesView {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl Render for EmployeesView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let employees = cx.background_executor().block({
            let state = self.state.clone();
            async move { state.get_filtered_employees().await }
        });

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_bold()
                    .child("Employees")
            )
            .child(
                div()
                    .grid()
                    .gap_4()
                    .children(employees.iter().map(|emp| self.render_employee_card(emp, cx)))
            )
    }
}

impl EmployeesView {
    fn render_employee_card(&self, employee: &Employee, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .p_4()
            .rounded_lg()
            .border_1()
            .border_color(rgb(0xe0e0e0))
            .bg(rgb(0xffffff))
            .hover(|style| style.bg(rgb(0xf5f5f5)))
            .child(
                div()
                    .text_lg()
                    .font_semibold()
                    .child(&employee.name)
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x666666))
                    .child(employee.role.map(|r| r.to_string()).unwrap_or_else(|| "No role".to_string()))
            )
            .child(
                div()
                    .text_sm()
                    .child(format!("Required days: {}", employee.required_days))
            )
    }
}

# crates/ui/src/components/schedules.rs
use crate::state::AppState;
use core::Weekday;
use gpui::*;

pub struct SchedulesView {
    state: AppState,
}

impl SchedulesView {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl Render for SchedulesView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let schedule = cx.background_executor().block({
            let state = self.state.clone();
            async move { state.get_schedule().await }
        });

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_bold()
                    .child("Schedule")
            )
            .child(match schedule {
                Some(s) => self.render_schedule(&s, cx).into_any_element(),
                None => div().child("No schedule generated").into_any_element(),
            })
    }
}

impl SchedulesView {
    fn render_schedule(&self, schedule: &core::Schedule, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let employees = cx.background_executor().block({
            let state = self.state.clone();
            async move { state.get_employees().await }
        });

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .gap_8()
                    .children(Weekday::all().iter().map(|day| {
                        let count = schedule.count_for_day(*day);
                        let emp_ids = schedule.get_employees_for_day(*day);
                        
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .p_4()
                            .border_1()
                            .rounded_lg()
                            .child(
                                div()
                                    .font_bold()
                                    .child(format!("{} ({})", day, count))
                            )
                            .children(emp_ids.iter().filter_map(|id| {
                                employees.iter().find(|e| e.id == *id).map(|emp| {
                                    div()
                                        .p_2()
                                        .rounded()
                                        .bg(rgb(0xf0f0f0))
                                        .child(&emp.name)
                                })
                            }))
                    }))
            )
    }
}

# crates/ui/src/components/settings.rs
use crate::state::AppState;
use gpui::*;

pub struct SettingsView {
    state: AppState,
}

impl SettingsView {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl Render for SettingsView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let config = cx.background_executor().block({
            let state = self.state.clone();
            async move { state.get_config().await }
        });

        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .text_2xl()
                    .font_bold()
                    .child("Settings")
            )
            .child(self.render_schedule_settings(&config, cx))
            .child(self.render_integration_settings(&config, cx))
            .child(self.render_keyboard_shortcuts(&config, cx))
    }
}

impl SettingsView {
    fn render_schedule_settings(&self, config: &core::AppConfig, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .border_1()
            .rounded_lg()
            .child(
                div()
                    .text_lg()
                    .font_semibold()
                    .child("Schedule Configuration")
            )
            .child(
                Checkbox::new("sex-distribution")
                    .label("Consider sex distribution")
                    .checked(config.schedule.consider_sex_distribution)
            )
            .child(
                Checkbox::new("role-distribution")
                    .label("Consider role distribution")
                    .checked(config.schedule.consider_role_distribution)
            )
    }

    fn render_integration_settings(&self, config: &core::AppConfig, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .border_1()
            .rounded_lg()
            .child(
                div()
                    .text_lg()
                    .font_semibold()
                    .child("Integrations")
            )
            .child(
                Checkbox::new("google-enabled")
                    .label("Enable Google Sheets")
                    .checked(config.integrations.google_enabled)
            )
            .child(
                Checkbox::new("discord-enabled")
                    .label("Enable Discord Notifications")
                    .checked(config.integrations.discord_enabled)
            )
    }

    fn render_keyboard_shortcuts(&self, config: &core::AppConfig, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .border_1()
            .rounded_lg()
            .child(
                div()
                    .text_lg()
                    .font_semibold()
                    .child("Keyboard Shortcuts")
            )
            .child(format!("Save: {}", config.shortcuts.save))
            .child(format!("Undo: {}", config.shortcuts.undo))
            .child(format!("Redo: {}", config.shortcuts.redo))
    }
}
