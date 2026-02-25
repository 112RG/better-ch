//! Dashboard view.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

use crate::state::{AppState, View};

/// Render the dashboard view.
pub fn render_dashboard(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    // Summary stats
    render_summary(frame, chunks[0], state);

    // Quick actions
    render_quick_actions(frame, chunks[1], state);

    // Recent alerts (placeholder)
    render_alerts(frame, chunks[2], state);
}

fn render_summary(frame: &mut Frame, area: Rect, state: &AppState) {
    let total_apps = state.applications.len();
    let running = state
        .applications
        .iter()
        .filter(|a| a.status == crate::state::AppStatus::Started)
        .count();
    let stopped = state
        .applications
        .iter()
        .filter(|a| a.status == crate::state::AppStatus::Stopped)
        .count();
    let failed = state
        .applications
        .iter()
        .filter(|a| a.status == crate::state::AppStatus::Failed)
        .count();

    let rows = vec![
        Row::new(vec!["Total", &total_apps.to_string()]),
        Row::new(vec!["Running", &running.to_string()])
            .style(Style::default().fg(Color::Green)),
        Row::new(vec!["Stopped", &stopped.to_string()])
            .style(Style::default().fg(Color::Red)),
        Row::new(vec!["Failed", &failed.to_string()])
            .style(Style::default().fg(Color::LightRed)),
    ];

    let table = Table::new(rows, [
            Constraint::Length(4),
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ])
        .block(Block::bordered(Borders::ALL).title("Application Status"))
        .widths([Constraint::Length(20), Constraint::Length(10)]);

    frame.render_widget(table, area);
}

fn render_quick_actions(frame: &mut Frame, area: Rect, _state: &AppState) {
    let actions = vec![
        "[s] Start Application",
        "[x] Stop Application",
        "[r] Restart Application",
        "[d] Delete Application",
        "[R] Refresh List",
        "[Enter] View Details",
        "[l] View Logs",
    ];

    let content = actions.join("\n");

    let paragraph = Paragraph::new(content)
        .block(Block::bordered(Borders::ALL).title("Quick Actions"))
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(paragraph, area);
}

fn render_alerts(frame: &mut Frame, area: Rect, _state: &AppState) {
    let paragraph = Paragraph::new("No recent alerts")
        .block(Block::bordered(Borders::ALL).title("Recent Alerts"));

    frame.render_widget(paragraph, area);
}