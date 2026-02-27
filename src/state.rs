//! Global application state management.
//!
//! Provides thread-safe state management for the TUI application.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::api::CloudHubClient;
use crate::config::Config;

/// Application state shared across the TUI.
///
/// Uses Arc<Mutex<>> for thread-safe access following the
/// Microsoft Rust guidelines for service types.
pub struct AppState {
    /// Configuration.
    pub config: Config,

    /// API client.
    pub client: CloudHubClient,

    /// Current view/tab.
    pub current_view: View,

    /// Selected application index (in app list).
    pub selected_app_index: usize,

    /// Applications list.
    pub applications: Vec<ApplicationSummary>,

    /// Loading state.
    pub is_loading: bool,

    /// Error message to display.
    pub error_message: Option<String>,

    /// Authentication status.
    pub is_authenticated: bool,
}

/// Application summary for list display.
#[derive(Debug, Clone)]
pub struct ApplicationSummary {
    /// Application name.
    pub name: String,

    /// Application domain.
    pub domain: String,

    /// Current status.
    pub status: AppStatus,

    /// Worker type.
    pub worker_type: String,

    /// Number of workers.
    pub worker_count: u32,

    /// CPU usage percentage.
    pub cpu_percent: f64,

    /// Memory usage in MB.
    pub memory_mb: u64,

    /// Runtime version.
    pub runtime_version: String,

    /// Last update time.
    pub last_update: Option<chrono::DateTime<chrono::Utc>>,
}

/// Application status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppStatus {
    /// Application is starting.
    Starting,

    /// Application is started/running.
    Started,

    /// Application is stopping.
    Stopping,

    /// Application is stopped.
    Stopped,

    /// Application is undeployed.
    Undeployed,

    /// Application deployment failed.
    Failed,

    /// Unknown status.
    Unknown,
}

impl AppStatus {
    /// Parse status from string.
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "STARTED" => Self::Started,
            "STARTING" => Self::Starting,
            "STOPPED" => Self::Stopped,
            "STOPPING" => Self::Stopping,
            "UNDEPLOYED" => Self::Undeployed,
            "FAILED" => Self::Failed,
            _ => Self::Unknown,
        }
    }
}

/// Available views/tabs in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    /// Dashboard overview.
    #[default]
    Dashboard,

    /// Applications list.
    Apps,

    /// Application details.
    AppDetail,

    /// Logs view.
    Logs,

    /// Settings view.
    Settings,
}

impl View {
    /// Get the display name for the view.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Apps => "Applications",
            Self::AppDetail => "App Details",
            Self::Logs => "Logs",
            Self::Settings => "Settings",
        }
    }

    /// Get the shortcut key for the view.
    pub fn shortcut(&self) -> char {
        match self {
            Self::Dashboard => '1',
            Self::Apps => '2',
            Self::AppDetail => '3',
            Self::Logs => '4',
            Self::Settings => '5',
        }
    }
}

impl AppState {
    /// Create a new application state.
    pub fn new(config: Config, client: CloudHubClient) -> Self {
        Self {
            config,
            client,
            current_view: View::Dashboard,
            selected_app_index: 0,
            applications: Vec::new(),
            is_loading: false,
            error_message: None,
            is_authenticated: false,
        }
    }

    /// Get the currently selected application.
    pub fn selected_app(&self) -> Option<&ApplicationSummary> {
        self.applications.get(self.selected_app_index)
    }

    /// Set error message.
    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error_message = Some(message.into());
    }

    /// Clear error message.
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Navigate to next view.
    pub fn next_view(&mut self) {
        self.current_view = match self.current_view {
            View::Dashboard => View::Apps,
            View::Apps => View::AppDetail,
            View::AppDetail => View::Logs,
            View::Logs => View::Settings,
            View::Settings => View::Dashboard,
        };
    }

    /// Navigate to previous view.
    pub fn prev_view(&mut self) {
        self.current_view = match self.current_view {
            View::Dashboard => View::Settings,
            View::Apps => View::Dashboard,
            View::AppDetail => View::Apps,
            View::Logs => View::AppDetail,
            View::Settings => View::Logs,
        };
    }
}

/// Thread-safe wrapper for application state.
pub type SharedState = Arc<Mutex<AppState>>;

/// Create a new shared state.
pub fn create_state(config: Config, client: CloudHubClient) -> SharedState {
    Arc::new(Mutex::new(AppState::new(config, client)))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod app_status {
        use super::*;

        #[test]
        fn test_from_str_started() {
            assert_eq!(AppStatus::from_str("started"), AppStatus::Started);
            assert_eq!(AppStatus::from_str("STARTED"), AppStatus::Started);
        }

        #[test]
        fn test_from_str_starting() {
            assert_eq!(AppStatus::from_str("starting"), AppStatus::Starting);
            assert_eq!(AppStatus::from_str("STARTING"), AppStatus::Starting);
        }

        #[test]
        fn test_from_str_stopped() {
            assert_eq!(AppStatus::from_str("stopped"), AppStatus::Stopped);
            assert_eq!(AppStatus::from_str("STOPPED"), AppStatus::Stopped);
        }

        #[test]
        fn test_from_str_stopping() {
            assert_eq!(AppStatus::from_str("stopping"), AppStatus::Stopping);
            assert_eq!(AppStatus::from_str("STOPPING"), AppStatus::Stopping);
        }

        #[test]
        fn test_from_str_undeployed() {
            assert_eq!(AppStatus::from_str("undeployed"), AppStatus::Undeployed);
            assert_eq!(AppStatus::from_str("UNDEPLOYED"), AppStatus::Undeployed);
        }

        #[test]
        fn test_from_str_failed() {
            assert_eq!(AppStatus::from_str("failed"), AppStatus::Failed);
            assert_eq!(AppStatus::from_str("FAILED"), AppStatus::Failed);
        }

        #[test]
        fn test_from_str_unknown() {
            assert_eq!(AppStatus::from_str("unknown"), AppStatus::Unknown);
            assert_eq!(AppStatus::from_str("random"), AppStatus::Unknown);
            assert_eq!(AppStatus::from_str(""), AppStatus::Unknown);
        }
    }

    mod view {
        use super::*;

        #[test]
        fn test_name() {
            assert_eq!(View::Dashboard.name(), "Dashboard");
            assert_eq!(View::Apps.name(), "Applications");
            assert_eq!(View::AppDetail.name(), "App Details");
            assert_eq!(View::Logs.name(), "Logs");
            assert_eq!(View::Settings.name(), "Settings");
        }

        #[test]
        fn test_shortcut() {
            assert_eq!(View::Dashboard.shortcut(), '1');
            assert_eq!(View::Apps.shortcut(), '2');
            assert_eq!(View::AppDetail.shortcut(), '3');
            assert_eq!(View::Logs.shortcut(), '4');
            assert_eq!(View::Settings.shortcut(), '5');
        }

        #[test]
        fn test_default() {
            assert_eq!(View::default(), View::Dashboard);
        }
    }

    mod app_state_navigation {
        use super::*;
        use crate::api::CloudHubClient;

        fn create_test_state() -> AppState {
            let config = Config::default();
            let client = CloudHubClient::new("https://test.example.com");
            AppState::new(config, client)
        }

        #[test]
        fn test_new() {
            let state = create_test_state();
            assert_eq!(state.current_view, View::Dashboard);
            assert_eq!(state.selected_app_index, 0);
            assert!(state.applications.is_empty());
            assert!(!state.is_loading);
            assert!(state.error_message.is_none());
            assert!(!state.is_authenticated);
        }

        #[test]
        fn test_selected_app_empty() {
            let state = create_test_state();
            assert!(state.selected_app().is_none());
        }

        #[test]
        fn test_selected_app_with_data() {
            let mut state = create_test_state();
            state.applications.push(ApplicationSummary {
                name: "test-app".to_string(),
                domain: "test-app.cloudhub.io".to_string(),
                status: AppStatus::Started,
                worker_type: "Micro".to_string(),
                worker_count: 1,
                cpu_percent: 0.5,
                memory_mb: 512,
                runtime_version: "4.5.0".to_string(),
                last_update: None,
            });

            let selected = state.selected_app().unwrap();
            assert_eq!(selected.name, "test-app");
        }

        #[test]
        fn test_set_and_clear_error() {
            let mut state = create_test_state();
            state.set_error("test error");
            assert_eq!(state.error_message, Some("test error".to_string()));

            state.clear_error();
            assert!(state.error_message.is_none());
        }

        #[test]
        fn test_navigation_next() {
            let mut state = create_test_state();
            assert_eq!(state.current_view, View::Dashboard);

            state.next_view();
            assert_eq!(state.current_view, View::Apps);

            state.next_view();
            assert_eq!(state.current_view, View::AppDetail);

            state.next_view();
            assert_eq!(state.current_view, View::Logs);

            state.next_view();
            assert_eq!(state.current_view, View::Settings);

            state.next_view();
            assert_eq!(state.current_view, View::Dashboard);
        }

        #[test]
        fn test_navigation_prev() {
            let mut state = create_test_state();
            assert_eq!(state.current_view, View::Dashboard);

            state.prev_view();
            assert_eq!(state.current_view, View::Settings);

            state.prev_view();
            assert_eq!(state.current_view, View::Logs);

            state.prev_view();
            assert_eq!(state.current_view, View::AppDetail);

            state.prev_view();
            assert_eq!(state.current_view, View::Apps);

            state.prev_view();
            assert_eq!(state.current_view, View::Dashboard);
        }
    }
}

// Rust guideline compliant 2026-02-21
