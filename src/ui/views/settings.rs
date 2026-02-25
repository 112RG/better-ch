//! Settings view.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::AppState;

/// Render the settings view.
pub fn render_settings(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    // Platform configuration
    let platform = Paragraph::new(format!(
        "Platform URL: {}",
        state.config.anypoint.platform_url
    ))
    .block(Block::bordered(Borders::ALL).title("Anypoint Platform"));

    // Environment
    let env = Paragraph::new(format!(
        "Environment: {}",
        state.config.anypoint.environment
    ))
    .block(Block::bordered(Borders::ALL).title("Environment"));

    // Authentication status
    let auth_status = if state.is_authenticated {
        "✓ Authenticated"
    } else {
        "✗ Not authenticated"
    };
    let auth_color = if state.is_authenticated {
        Color::Green
    } else {
        Color::Red
    };
    let auth = Paragraph::new(auth_status)
        .block(Block::bordered(Borders::ALL).title("Authentication"))
        .style(Style::default().fg(auth_color));

    // Configuration help
    let config_help = Paragraph::new(
        "Configure credentials via environment variables:\n\
         • ANYPOINT_CLIENT_ID - OAuth2 client ID\n\
         • ANYPOINT_CLIENT_SECRET - OAuth2 client secret\n\
         • ANYPOINT_PLATFORM_URL - Platform URL (default: https://anypoint.mulesoft.com)\n\
         • ANYPOINT_ENVIRONMENT - Environment name (default: production)",
    )
    .block(Block::bordered(Borders::ALL).title("Environment Variables"));

    // UI settings
    let ui_settings = Paragraph::new(format!(
        "Refresh Interval: {} seconds\n\
         Log Buffer Size: {} lines\n\
         Debug Mode: {}",
        state.config.ui.refresh_interval,
        state.config.ui.log_buffer_size,
        if state.config.ui.debug { "Enabled" } else { "Disabled" }
    ))
    .block(Block::bordered(Borders::ALL).title("UI Settings"));

    frame.render_widget(platform, chunks[0]);
    frame.render_widget(env, chunks[1]);
    frame.render_widget(auth, chunks[2]);
    frame.render_widget(config_help, chunks[3]);
    frame.render_widget(ui_settings, chunks[4]);
}