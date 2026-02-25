//! Navigation module for TUI views.

use crate::state::View;

/// Navigation handler for the TUI.
pub struct Navigation;

impl Navigation {
    /// Handle navigation input.
    pub fn handle_key(view: View, key: char) -> Option<View> {
        match key {
            '1' => Some(View::Dashboard),
            '2' => Some(View::Apps),
            '3' => Some(View::AppDetail),
            '4' => Some(View::Logs),
            '5' => Some(View::Settings),
            _ => None,
        }
    }

    /// Get all available views.
    pub fn all_views() -> Vec<View> {
        vec![
            View::Dashboard,
            View::Apps,
            View::AppDetail,
            View::Logs,
            View::Settings,
        ]
    }
}