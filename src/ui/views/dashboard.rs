//! Dashboard view.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::state::AppState;

pub fn render_dashboard(frame: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);

    render_summary(frame, chunks[0], state);
    render_quick_actions(frame, chunks[1], state);
}

fn render_summary(frame: &mut Frame, area: Rect, state: &AppState) {
    let total = state.applications.len();
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

    let content = format!(
        "  Total Applications: {}\n  Running: {}\n  Stopped: {}\n  Failed: {}",
        total, running, stopped, failed
    );

    let paragraph = Paragraph::new(content)
        .block(Block::bordered().title("Application Status"))
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, area);
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

    let items: Vec<ListItem> = actions.iter().map(|a| ListItem::new(*a)).collect();

    let list = List::new(items)
        .block(Block::bordered().title("Quick Actions"))
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(list, area);
}
