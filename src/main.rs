//! CloudHub Runtime Manager - A TUI for managing CloudHub applications.
//!
//! This application provides a terminal user interface for managing
//! MuleSoft CloudHub applications via the Anypoint Platform API.

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api::CloudHubClient;
use crate::config::Config;
use crate::state::{create_state, AppState, SharedState};
use crate::ui::TuiApp;

/// Application entry point.
fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting CloudHub Runtime Manager");

    // Load configuration
    let config = match Config::load_with_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::warn!("Failed to load config, using defaults: {}", e);
            Config::default()
        }
    };

    tracing::info!(
        "Connected to: {}",
        config.anypoint.platform_url
    );

    // Create API client
    let client = CloudHubClient::new(&config.cloudhub_url());

    // Create shared state
    let state: SharedState = create_state(config, client);

    // Run the TUI application
    let mut app = TuiApp::new(state);

    if let Err(e) = app.run() {
        tracing::error!("Application error: {}", e);
        std::process::exit(1);
    }

    tracing::info!("Shutting down");
    Ok(())
}

// Re-export modules for convenience
mod api;
mod auth;
mod config;
mod error;
mod state;
mod ui;
