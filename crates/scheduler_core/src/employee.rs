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
            return Err(crate::CoreError::InvalidEmployee(
                "Name cannot be empty".to_string(),
            ));
        }

        if self.required_days > 5 {
            return Err(crate::CoreError::InvalidEmployee(
                "Required days cannot exceed 5".to_string(),
            ));
        }

        if self.is_mentee && self.mentor_id.is_none() {
            return Err(crate::CoreError::InvalidEmployee(
                "Mentee must have a mentor".to_string(),
            ));
        }

        if !self.is_mentee && self.mentor_id.is_some() {
            return Err(crate::CoreError::InvalidEmployee(
                "Non-mentee cannot have mentor".to_string(),
            ));
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
