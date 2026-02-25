//! Error types for the CloudHub Runtime Manager application.
//!
//! This module provides application-level error handling using anyhow.

use thiserror::Error;

/// Main application error type.
///
/// Errors are categorized into authentication, API, configuration, and UI errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Authentication-related errors.
    #[error("authentication error: {0}")]
    Auth(#[from] AuthError),

    /// API-related errors.
    #[error("API error: {0}")]
    Api(#[from] ApiError),

    /// Configuration-related errors.
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    /// UI-related errors.
    #[error("UI error: {0}")]
    Ui(#[from] UiError),

    /// I/O errors.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Authentication errors.
#[derive(Debug, Error)]
pub enum AuthError {
    /// Failed to obtain access token.
    #[error("failed to obtain access token: {0}")]
    TokenFetch(String),

    /// Invalid credentials.
    #[error("invalid credentials")]
    InvalidCredentials,

    /// Token storage error.
    #[error("failed to store token: {0}")]
    TokenStorage(String),

    /// Token expired.
    #[error("token expired, please re-authenticate")]
    TokenExpired,

    /// Missing credentials.
    #[error("missing credentials: {0}")]
    MissingCredentials(String),
}

/// API-related errors.
#[derive(Debug, Error)]
pub enum ApiError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Request(String),

    /// Invalid response from server.
    #[error("invalid response: {0}")]
    InvalidResponse(String),

    /// Application not found.
    #[error("application not found: {0}")]
    NotFound(String),

    /// Server returned an error.
    #[error("server error ({status}): {message}")]
    ServerError { status: u16, message: String },

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Configuration errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Missing required configuration.
    #[error("missing configuration: {0}")]
    Missing(String),

    /// Invalid configuration value.
    #[error("invalid configuration: {0}")]
    Invalid(String),

    /// Failed to read/write config file.
    #[error("config file error: {0}")]
    File(String),
}

/// UI-related errors.
#[derive(Debug, Error)]
pub enum UiError {
    /// Terminal initialization failed.
    #[error("failed to initialize terminal: {0}")]
    TerminalInit(String),

    /// Render error.
    #[error("render error: {0}")]
    Render(String),

    /// Invalid input.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

// Implement anyhow::Context for convenient error handling.
impl<T> From<Error> for anyhow::Result<T> {
    fn from(err: Error) -> Self {
        Err(err.into())
    }
}