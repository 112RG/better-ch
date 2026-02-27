//! Settings view.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph},
};

use crate::state::AppState;

pub fn render_settings(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(5),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    let platform = Paragraph::new(format!(
        "Platform URL: {}",
        state.config.anypoint.platform_url
    ))
    .block(Block::bordered().title("Anypoint Platform"));

    let env = Paragraph::new(format!(
        "Environment: {}",
        state.config.anypoint.environment
    ))
    .block(Block::bordered().title("Environment"));

    // Authentication status with login button hint
    let (auth_status, auth_hint) = if state.is_authenticated {
        ("✓ Authenticated", "Press [L] to logout")
    } else {
        ("✗ Not authenticated", "Press [L] to login with SSO")
    };
    let auth_color = if state.is_authenticated {
        Color::Green
    } else {
        Color::Yellow
    };
    let auth = Paragraph::new(format!("{}\n{}", auth_status, auth_hint))
        .block(Block::bordered().title("Authentication"))
        .style(Style::default().fg(auth_color));

    let config_help = Paragraph::new(
        "Configure credentials via environment variables:\n\
         • ANYPOINT_CLIENT_ID - OAuth2 client ID\n\
         • ANYPOINT_CLIENT_SECRET - OAuth2 client secret\n\
         • ANYPOINT_PLATFORM_URL - Platform URL\n\
         • ANYPOINT_ENVIRONMENT - Environment name\n\n\
         Or use the VSCode extension credentials:\n\
         client_id: a7db79120339458da2d7ba979ee94a42\n\
         client_secret: 339A336DA32446dFb8B2945400E607B8",
    )
    .block(Block::bordered().title("Configuration"));

    let ui_settings = Paragraph::new(format!(
        "Refresh Interval: {} seconds\nLog Buffer Size: {} lines\nDebug Mode: {}",
        state.config.ui.refresh_interval,
        state.config.ui.log_buffer_size,
        if state.config.ui.debug {
            "Enabled"
        } else {
            "Disabled"
        }
    ))
    .block(Block::bordered().title("UI Settings"));

    frame.render_widget(platform, chunks[0]);
    frame.render_widget(env, chunks[1]);
    frame.render_widget(auth, chunks[2]);
    frame.render_widget(config_help, chunks[3]);
    frame.render_widget(ui_settings, chunks[4]);
}
