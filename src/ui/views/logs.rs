//! Logs view.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::AppState;

/// Render the logs view.
pub fn render_logs(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Header
    let app_name = state
        .selected_app()
        .map(|a| a.name.clone())
        .unwrap_or_else(|| "No application".to_string());

    let header = Paragraph::new(format!("Logs for: {}", app_name))
        .block(Block::bordered(Borders::ALL).title("Application Logs"));

    frame.render_widget(header, chunks[0]);

    // Log content placeholder
    let content = Paragraph::new(
        "Select an application and press 'l' to view logs.\n\n\
         Use arrow keys to scroll through log entries.\n\n\
         Log levels: INFO (white), WARN (yellow), ERROR (red), DEBUG (gray)",
    )
    .block(Block::bordered(Borders::ALL))
    .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(content, chunks[1]);
}