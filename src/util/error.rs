//! Error types and result aliases for Yoru.

use thiserror::Error;

/// Application-level error type.
#[derive(Debug, Error)]
pub enum YoruError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML serialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Script error: {0}")]
    Script(String),
}

/// Convenience alias used across modules.
pub type YoruResult<T> = Result<T, YoruError>;
