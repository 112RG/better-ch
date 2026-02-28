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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_auth() {
        let err = Error::Auth(AuthError::TokenExpired);
        assert_eq!(
            format!("{}", err),
            "authentication error: token expired, please re-authenticate"
        );
    }

    #[test]
    fn test_error_display_api() {
        let err = Error::Api(ApiError::NotFound("app-name".to_string()));
        assert_eq!(
            format!("{}", err),
            "API error: application not found: app-name"
        );
    }

    #[test]
    fn test_error_display_config() {
        let err = Error::Config(ConfigError::Missing("client_id".to_string()));
        assert_eq!(
            format!("{}", err),
            "configuration error: missing configuration: client_id"
        );
    }

    #[test]
    fn test_error_debug() {
        let err = Error::Auth(AuthError::TokenExpired);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Auth"));
        assert!(debug_str.contains("TokenExpired"));
    }

    #[test]
    fn test_auth_error_variants() {
        let err = AuthError::TokenFetch("network error".to_string());
        assert_eq!(
            format!("{}", err),
            "failed to obtain access token: network error"
        );

        let err = AuthError::InvalidCredentials;
        assert_eq!(format!("{}", err), "invalid credentials");

        let err = AuthError::TokenStorage("keyring error".to_string());
        assert_eq!(format!("{}", err), "failed to store token: keyring error");

        let err = AuthError::TokenExpired;
        assert_eq!(format!("{}", err), "token expired, please re-authenticate");

        let err = AuthError::MissingCredentials("client_id".to_string());
        assert_eq!(format!("{}", err), "missing credentials: client_id");
    }

    #[test]
    fn test_api_error_variants() {
        let err = ApiError::Request("timeout".to_string());
        assert_eq!(format!("{}", err), "HTTP request failed: timeout");

        let err = ApiError::InvalidResponse("invalid JSON".to_string());
        assert_eq!(format!("{}", err), "invalid response: invalid JSON");

        let err = ApiError::NotFound("my-app".to_string());
        assert_eq!(format!("{}", err), "application not found: my-app");

        let err = ApiError::ServerError {
            status: 500,
            message: "Internal error".to_string(),
        };
        assert_eq!(format!("{}", err), "server error (500): Internal error");
    }

    #[test]
    fn test_config_error_variants() {
        let err = ConfigError::Missing("api_key".to_string());
        assert_eq!(format!("{}", err), "missing configuration: api_key");

        let err = ConfigError::Invalid("invalid value".to_string());
        assert_eq!(format!("{}", err), "invalid configuration: invalid value");

        let err = ConfigError::File("read error".to_string());
        assert_eq!(format!("{}", err), "config file error: read error");
    }

    #[test]
    fn test_ui_error_variants() {
        let err = UiError::TerminalInit("no terminal".to_string());
        assert_eq!(
            format!("{}", err),
            "failed to initialize terminal: no terminal"
        );

        let err = UiError::Render("draw error".to_string());
        assert_eq!(format!("{}", err), "render error: draw error");

        let err = UiError::InvalidInput("empty input".to_string());
        assert_eq!(format!("{}", err), "invalid input: empty input");
    }
}

// Rust guideline compliant 2026-02-21
