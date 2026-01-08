mod csv_export;
mod csv_import;
mod json_export;
mod json_import;

pub use csv_export::CsvExporter;
pub use csv_import::CsvImporter;
pub use json_export::JsonExporter;
pub use json_import::JsonImporter;

#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Integration error: {0}")]
    GeneralError(String),
}

pub type Result<T> = std::result::Result<T, IntegrationError>;
