//! Configuration management for CloudHub Runtime Manager.
//!
//! Handles loading configuration from file, environment variables, and defaults.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{ConfigError, Error};

/// Application configuration.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// Anypoint Platform configuration.
    pub anypoint: AnypointConfig,

    /// UI configuration.
    pub ui: UiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            anypoint: AnypointConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

/// Anypoint Platform configuration.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct AnypointConfig {
    /// Platform URL (e.g., https://anypoint.mulesoft.com).
    pub platform_url: String,

    /// Business group ID (optional).
    pub business_group_id: Option<String>,

    /// Environment name.
    pub environment: String,

    /// Client ID for OAuth2.
    #[serde(skip_serializing)]
    pub client_id: Option<String>,

    /// Client secret for OAuth2.
    #[serde(skip_serializing)]
    pub client_secret: Option<String>,
}

impl Default for AnypointConfig {
    fn default() -> Self {
        Self {
            platform_url: "https://anypoint.mulesoft.com".to_string(),
            business_group_id: None,
            environment: "production".to_string(),
            client_id: None,
            client_secret: None,
        }
    }
}

/// UI configuration.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct UiConfig {
    /// Refresh interval in seconds.
    pub refresh_interval: u64,

    /// Number of log lines to keep.
    pub log_buffer_size: usize,

    /// Show debug information.
    pub debug: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            refresh_interval: 30,
            log_buffer_size: 1000,
            debug: false,
        }
    }
}

impl Config {
    /// Get the default configuration file path.
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("better-ch")
            .join("config.toml")
    }

    /// Load configuration from file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let path = path.into();
        if !path.exists() {
            return Ok(Config::default());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| Error::Config(ConfigError::File(e.to_string())))?;

        toml::from_str(&content).map_err(|e| Error::Config(ConfigError::Invalid(e.to_string())))
    }

    /// Save configuration to file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be written.
    pub fn save(&self, path: impl Into<PathBuf>) -> Result<(), Error> {
        let path = path.into();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(ConfigError::Invalid(e.to_string())))?;

        std::fs::write(&path, content)
            .map_err(|e| Error::Config(ConfigError::File(e.to_string())))?;

        Ok(())
    }

    /// Load configuration with environment variable overrides.
    ///
    /// Environment variables:
    /// - `ANYPOINT_CLIENT_ID`: OAuth2 client ID
    /// - `ANYPOINT_CLIENT_SECRET`: OAuth2 client secret
    /// - `ANYPOINT_PLATFORM_URL`: Platform URL
    /// - `ANYPOINT_ENVIRONMENT`: Environment name
    pub fn load_with_env() -> Result<Self, Error> {
        let mut config = Self::load(Self::config_path())?;

        // Override with environment variables
        if let Ok(client_id) = std::env::var("ANYPOINT_CLIENT_ID") {
            config.anypoint.client_id = Some(client_id);
        }

        if let Ok(client_secret) = std::env::var("ANYPOINT_CLIENT_SECRET") {
            config.anypoint.client_secret = Some(client_secret);
        }

        if let Ok(platform_url) = std::env::var("ANYPOINT_PLATFORM_URL") {
            config.anypoint.platform_url = platform_url;
        }

        if let Ok(environment) = std::env::var("ANYPOINT_ENVIRONMENT") {
            config.anypoint.environment = environment;
        }

        Ok(config)
    }

    /// Get the API base URL for CloudHub.
    pub fn cloudhub_url(&self) -> String {
        format!("{}/cloudhub/api/v1", self.anypoint.platform_url)
    }

    /// Check if credentials are configured.
    pub fn has_credentials(&self) -> bool {
        self.anypoint.client_id.is_some() && self.anypoint.client_secret.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(
            config.anypoint.platform_url,
            "https://anypoint.mulesoft.com"
        );
        assert_eq!(config.anypoint.environment, "production");
        assert!(config.anypoint.business_group_id.is_none());
        assert_eq!(config.ui.refresh_interval, 30);
        assert_eq!(config.ui.log_buffer_size, 1000);
        assert!(!config.ui.debug);
    }

    #[test]
    fn test_config_load_nonexistent_file() {
        let config = Config::load("/nonexistent/path/config.toml").unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_config_load_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let toml_content = r#"
[anypoint]
platform-url = "https://custom.example.com"
business-group-id = "test-org"
environment = "sandbox"

[ui]
refresh-interval = 60
log-buffer-size = 500
debug = true
"#;
        std::fs::write(&config_path, toml_content).unwrap();

        let config = Config::load(&config_path).unwrap();

        assert_eq!(config.anypoint.platform_url, "https://custom.example.com");
        assert_eq!(
            config.anypoint.business_group_id,
            Some("test-org".to_string())
        );
        assert_eq!(config.anypoint.environment, "sandbox");
        assert_eq!(config.ui.refresh_interval, 60);
        assert_eq!(config.ui.log_buffer_size, 500);
        assert!(config.ui.debug);
    }

    #[test]
    fn test_config_load_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        std::fs::write(&config_path, "invalid toml content = ").unwrap();

        let result = Config::load(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let mut config = Config::default();
        config.anypoint.platform_url = "https://saved.example.com".to_string();
        config.anypoint.business_group_id = Some("saved-org".to_string());
        config.ui.debug = true;

        config.save(&config_path).unwrap();

        let loaded = Config::load(&config_path).unwrap();
        assert_eq!(loaded.anypoint.platform_url, "https://saved.example.com");
        assert_eq!(
            loaded.anypoint.business_group_id,
            Some("saved-org".to_string())
        );
        assert!(loaded.ui.debug);
    }

    #[test]
    fn test_cloudhub_url() {
        let config = Config::default();
        assert_eq!(
            config.cloudhub_url(),
            "https://anypoint.mulesoft.com/cloudhub/api/v1"
        );

        let mut config = Config::default();
        config.anypoint.platform_url = "https://custom.platform.com".to_string();
        assert_eq!(
            config.cloudhub_url(),
            "https://custom.platform.com/cloudhub/api/v1"
        );
    }

    #[test]
    fn test_has_credentials() {
        let mut config = Config::default();
        assert!(!config.has_credentials());

        config.anypoint.client_id = Some("test-id".to_string());
        assert!(!config.has_credentials());

        config.anypoint.client_secret = Some("test-secret".to_string());
        assert!(config.has_credentials());
    }
}

// Rust guideline compliant 2026-02-21
