use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Employee {
    pub id: usize,
    pub name: String,
    pub sex: Sex,
    pub role: Role,
    pub required_days: u8,
    pub fixed_days: Vec<Weekday>,
    pub is_nsp: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum Role {
    HR,
    AiLlmEngineer,
    SocialMediaMarketing,
    // MarketingManager,
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
            // Role::MarketingManager => write!(f, "Marketing Manager"),
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
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

// type definitions for scheduler logic
pub type MonthlySchedule = HashMap<Weekday, Vec<Employee>>;
pub type DayCount = HashMap<Weekday, usize>;
pub type PastSchedules = HashMap<usize, Vec<HashSet<Weekday>>>;

// Day combinations for different required office days
#[derive(Debug, Clone)]
pub struct DayCombination {
    pub days: Vec<Weekday>,
}

impl DayCombination {
    fn new(days: Vec<Weekday>) -> Self {
        Self { days }
    }
}

pub struct ScheduleGenerator {
    pub weekdays: Vec<Weekday>,
    pub day_combinations: HashMap<usize, Vec<DayCombination>>,
}

impl ScheduleGenerator {
    pub fn new() -> Self {
        let weekdays = vec![
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
        ];

        let day_combinations = Self::initialize_day_combinations();

        Self {
            weekdays,
            day_combinations,
        }
    }

    fn initialize_day_combinations() -> HashMap<usize, Vec<DayCombination>> {
        let mut combinations = HashMap::new();

        // 1-day combinations
        combinations.insert(
            1,
            vec![
                DayCombination::new(vec![Weekday::Monday]),
                DayCombination::new(vec![Weekday::Tuesday]),
                DayCombination::new(vec![Weekday::Wednesday]),
                DayCombination::new(vec![Weekday::Thursday]),
                DayCombination::new(vec![Weekday::Friday]),
            ],
        );

        // 2-day combinations
        combinations.insert(
            2,
            vec![
                DayCombination::new(vec![Weekday::Monday, Weekday::Wednesday]),
                DayCombination::new(vec![Weekday::Monday, Weekday::Thursday]),
                DayCombination::new(vec![Weekday::Monday, Weekday::Friday]),
                DayCombination::new(vec![Weekday::Tuesday, Weekday::Thursday]),
                DayCombination::new(vec![Weekday::Tuesday, Weekday::Friday]),
                DayCombination::new(vec![Weekday::Wednesday, Weekday::Friday]),
            ],
        );

        // 3-day combinations
        combinations.insert(
            3,
            vec![DayCombination::new(vec![
                Weekday::Monday,
                Weekday::Wednesday,
                Weekday::Friday,
            ])],
        );

        // 5-day combinations
        combinations.insert(
            5,
            vec![DayCombination::new(vec![
                Weekday::Monday,
                Weekday::Tuesday,
                Weekday::Wednesday,
                Weekday::Thursday,
                Weekday::Friday,
            ])],
        );

        combinations
    }
}

// #[derive(Debug, Clone)]
// pub struct ScheduleStatistics {
//     pub day_counts: HashMap<Weekday, usize>,
//     pub gender_distribution: HashMap<Weekday, HashMap<String, usize>>,
//     pub role_distribution: HashMap<Weekday, HashMap<String, usize>>,
//     pub total_employees: usize,
//     pub average_daily_attendance: f64,
// }
