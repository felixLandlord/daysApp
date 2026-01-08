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
