//! Error types for the browserware ecosystem.

use thiserror::Error;

/// Result type alias using browserware's Error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in browserware operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Browser was not found
    #[error("browser not found: {0}")]
    BrowserNotFound(String),

    /// Profile was not found
    #[error("profile not found: {0}")]
    ProfileNotFound(String),

    /// Configuration error
    #[error("configuration error: {0}")]
    Config(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// URL parsing error
    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML parsing error
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Platform not supported
    #[error("platform not supported: {0}")]
    UnsupportedPlatform(String),

    /// Generic error with message
    #[error("{0}")]
    Other(String),
}
