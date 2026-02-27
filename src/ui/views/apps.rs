//! Applications list view.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::state::{AppState, AppStatus};

pub fn render_apps(frame: &mut Frame, area: Rect, state: &AppState) {
    if state.applications.is_empty() {
        render_empty(frame, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    render_app_list(frame, chunks[0], state);
    render_help(frame, chunks[1]);
}

fn render_empty(frame: &mut Frame, area: Rect) {
    let paragraph = Paragraph::new("No applications found.\n\nPress 'R' to refresh the list.")
        .block(Block::bordered().title("Applications"))
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn render_app_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let items: Vec<ListItem> = state
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

            let line = format!(
                "{:4} | {:20} | {:10} | {:12} | CPU: {:5.1}% | RAM: {}MB",
                i + 1,
                app.name,
                format!("{:?}", app.status),
                format!("{} x {}", app.worker_count, app.worker_type),
                app.cpu_percent,
                app.memory_mb
            );

            ListItem::new(line).style(Style::default().fg(status_color))
        })
        .collect();

    let list = List::new(items)
        .block(Block::bordered().title("Applications"))
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(list, area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let paragraph = Paragraph::new(
        "[s] Start | [x] Stop | [r] Restart | [d] Delete | [Enter] Details | [R] Refresh",
    )
    .block(Block::bordered().title("Actions"))
    .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}
