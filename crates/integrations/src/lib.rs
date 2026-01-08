pub mod discord;
pub mod google;

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
