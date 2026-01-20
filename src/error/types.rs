//! Core error types using thiserror for structured error handling.

use std::time::Duration;
use thiserror::Error;

/// Main error type for basevn-migro operations.
#[derive(Debug, Error)]
pub enum MigroError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Extract error: {0}")]
    Extract(#[from] ExtractError),

    #[error("Transform error: {0}")]
    Transform(#[from] TransformError),

    #[error("Load error: {0}")]
    Load(#[from] LoadError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

/// Configuration-related errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to load config from {path}: {source}")]
    FileRead {
        path: String,
        source: std::io::Error,
    },

    #[error("Invalid YAML config: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("Invalid JSON config: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Missing required field: {field}")]
    MissingField { field: String },
}

/// Data extraction errors.
#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("Failed to read source file: {path}")]
    FileRead {
        path: String,
        source: std::io::Error,
    },

    #[error("Invalid CSV format at line {line}: {message}")]
    CsvParse { line: usize, message: String },

    #[error("Invalid Excel format: {0}")]
    ExcelParse(String),

    #[error("API request failed: {0}")]
    ApiRequest(String),

    #[error("Unsupported source type: {0}")]
    UnsupportedType(String),
}

/// Data transformation errors.
#[derive(Debug, Error)]
pub enum TransformError {
    #[error("Missing required field: {field}")]
    MissingRequiredField { field: String },

    #[error("Invalid field value for '{field}': {message}")]
    InvalidValue { field: String, message: String },

    #[error("Transformation failed: {0}")]
    TransformFailed(String),
}

/// Data loading errors.
#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Connection failed: {0}")]
    Connection(String),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("API request failed: {0}")]
    ApiRequest(String),

    #[error("Rate limit exceeded")]
    RateLimit {
        /// Server-provided retry delay (from Retry-After header)
        retry_after: Option<Duration>,
    },

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

/// Record-level error with context.
#[derive(Debug, Clone)]
pub struct RecordError {
    pub offset: usize,
    pub key: Option<String>,
    pub error: String,
}
