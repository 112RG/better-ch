//! Applications list view.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Row, Table},
    Frame,
};

use crate::state::{AppState, AppStatus};

/// Render the applications list view.
pub fn render_apps(frame: &mut Frame, area: Rect, state: &AppState) {
    if state.applications.is_empty() {
        render_empty(frame, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    render_app_table(frame, chunks[0], state);
    render_help(frame, chunks[1]);
}

fn render_empty(frame: &mut Frame, area: Rect) {
    let paragraph = ratatui::widgets::Paragraph::new(
        "No applications found.\n\nPress 'R' to refresh the list.",
    )
    .block(Block::bordered(Borders::ALL).title("Applications"))
    .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_app_table(frame: &mut Frame, area: Rect, state: &AppState) {
    let rows: Vec<Row> = state
        .applications
        .iter()
        .enumerate()
        .map(|(i, app)| {
            let status_color = match app.status {
                AppStatus::Started => Color::Green,
                AppStatus::Stopped => Color::Red,
                AppStatus::Starting | AppStatus::Stopping => Color::Yellow,
                AppStatus::Failed => Color::LightRed,
                AppStatus::Undeployed => Color::DarkGray,
                AppStatus::Unknown => Color::White,
            };

            Row::new(vec![
                format!("{}", i + 1),
                app.name.clone(),
                format!("{:?}", app.status),
                format!("{} x {}", app.worker_count, app.worker_type),
                format!("{:.1}%", app.cpu_percent),
                format!("{} MB", app.memory_mb),
                app.runtime_version.clone(),
            ])
            .style(Style::default().fg(status_color))
        })
        .collect();

    let table = Table::new(rows, [
            Constraint::Length(4),
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ])
        .block(Block::bordered(Borders::ALL).title("Applications"))
        .header(
            Row::new(vec!["#", "Name", "Status", "Workers", "CPU", "RAM", "Runtime"])
                .style(Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)),
        )
        .widths([
            Constraint::Length(4),
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(15),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(12),
        ])
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(table, area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let paragraph = ratatui::widgets::Paragraph::new(
        "[s] Start | [x] Stop | [r] Restart | [d] Delete | [Enter] Details | [R] Refresh",
    )
    .block(Block::bordered(Borders::ALL).title("Actions"))
    .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}