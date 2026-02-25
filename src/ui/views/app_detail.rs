//! Application detail view.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

use crate::state::{AppState, AppStatus};

/// Render the application detail view.
pub fn render_app_detail(frame: &mut Frame, area: Rect, state: &AppState) {
    let app = match state.selected_app() {
        Some(app) => app,
        None => {
            render_no_selection(frame, area);
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(area);

    // Application name
    let name = Paragraph::new(app.name.clone())
        .block(Block::bordered(Borders::ALL).title("Application Name"))
        .style(Style::default().fg(Color::Cyan));

    // Status
    let status_color = match app.status {
        AppStatus::Started => Color::Green,
        AppStatus::Stopped => Color::Red,
        AppStatus::Starting | AppStatus::Stopping => Color::Yellow,
        AppStatus::Failed => Color::LightRed,
        _ => Color::White,
    };
    let status = Paragraph::new(format!("{:?}", app.status))
        .block(Block::bordered(Borders::ALL).title("Status"))
        .style(Style::default().fg(status_color));

    // Resources
    let resources = Paragraph::new(format!(
        "Workers: {} x {}\nCPU: {:.1}%\nRAM: {} MB",
        app.worker_count, app.worker_type, app.cpu_percent, app.memory_mb
    ))
    .block(Block::bordered(Borders::ALL).title("Resources"));

    // Runtime info
    let runtime = Paragraph::new(format!(
        "Domain: {}\nRuntime: {}\nLast Update: {:?}",
        app.domain,
        app.runtime_version,
        app.last_update
    ))
    .block(Block::bordered(Borders::ALL).title("Runtime Information"));

    // Actions
    let actions = Paragraph::new(
        "[s] Start | [x] Stop | [r] Restart | [d] Delete | [l] Logs | [b] Back",
    )
    .block(Block::bordered(Borders::ALL).title("Actions"))
    .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(name, chunks[0]);
    frame.render_widget(status, chunks[1]);
    frame.render_widget(resources, chunks[2]);
    frame.render_widget(runtime, chunks[3]);
    frame.render_widget(actions, chunks[4]);
}

fn render_no_selection(frame: &mut Frame, area: Rect) {
    let paragraph = Paragraph::new(
        "No application selected.\n\nGo to Applications tab and select one.",
    )
    .block(Block::bordered(Borders::ALL).title("App Details"))
    .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}