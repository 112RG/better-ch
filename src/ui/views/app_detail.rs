//! Application detail view.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::state::{AppState, AppStatus};

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
            Constraint::Min(0),
        ])
        .split(area);

    let status_color = match app.status {
        AppStatus::Started => Color::Green,
        AppStatus::Stopped => Color::Red,
        AppStatus::Starting | AppStatus::Stopping => Color::Yellow,
        AppStatus::Failed => Color::LightRed,
        _ => Color::White,
    };

    let name = Paragraph::new(app.name.clone())
        .block(Block::bordered().title("Application Name"))
        .style(Style::default().fg(Color::Cyan));

    let status = Paragraph::new(format!("{:?}", app.status))
        .block(Block::bordered().title("Status"))
        .style(Style::default().fg(status_color));

    let resources = Paragraph::new(format!(
        "Workers: {} x {}\nCPU: {:.1}%\nRAM: {} MB\nRuntime: {}",
        app.worker_count, app.worker_type, app.cpu_percent, app.memory_mb, app.runtime_version
    ))
    .block(Block::bordered().title("Resources"));

    let actions =
        Paragraph::new("[s] Start | [x] Stop | [r] Restart | [d] Delete | [l] Logs | [b] Back")
            .block(Block::bordered().title("Actions"))
            .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(name, chunks[0]);
    frame.render_widget(status, chunks[1]);
    frame.render_widget(resources, chunks[2]);
    frame.render_widget(actions, chunks[3]);
}

fn render_no_selection(frame: &mut Frame, area: Rect) {
    let paragraph =
        Paragraph::new("No application selected.\n\nGo to Applications tab and select one.")
            .block(Block::bordered().title("App Details"))
            .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}
