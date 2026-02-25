//! Main TUI application.

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};
use std::io;

use crate::state::{AppState, SharedState, View};

/// Main TUI application.
pub struct TuiApp {
    pub state: SharedState,
}

impl TuiApp {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        terminal.clear()?;

        loop {
            let state = self.state.blocking_lock();
            terminal.draw(|f| self.render(f, &state))?;
            drop(state);

            use ratatui::crossterm::event::KeyCode;
            if let Ok(event) = ratatui::crossterm::event::read() {
                if let ratatui::crossterm::event::Event::Key(key) = event {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('1') => self.state.blocking_lock().current_view = View::Dashboard,
                        KeyCode::Char('2') => self.state.blocking_lock().current_view = View::Apps,
                        KeyCode::Char('3') => self.state.blocking_lock().current_view = View::AppDetail,
                        KeyCode::Char('4') => self.state.blocking_lock().current_view = View::Logs,
                        KeyCode::Char('5') => self.state.blocking_lock().current_view = View::Settings,
                        KeyCode::Tab => self.state.blocking_lock().next_view(),
                        KeyCode::BackTab => self.state.blocking_lock().prev_view(),
                        KeyCode::Down | KeyCode::Char('j') => {
                            let mut s = self.state.blocking_lock();
                            if s.selected_app_index < s.applications.len().saturating_sub(1) {
                                s.selected_app_index += 1;
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let mut s = self.state.blocking_lock();
                            if s.selected_app_index > 0 {
                                s.selected_app_index -= 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
            .split(frame.area());

        self.render_tabs(frame, chunks[0], state);
        self.render_content(frame, chunks[1], state);
        self.render_status_bar(frame, chunks[2], state);
    }

    fn render_tabs(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let titles = vec!["Dashboard", "Applications", "App Details", "Logs", "Settings"];
        let views = [View::Dashboard, View::Apps, View::AppDetail, View::Logs, View::Settings];
        let selected = views.iter().position(|v| *v == state.current_view).unwrap_or(0);

        let tabs = Tabs::new(titles)
            .block(Block::bordered(Borders::ALL).title("CloudHub Runtime Manager"))
            .select(selected)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

        frame.render_widget(tabs, area);
    }

    fn render_content(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        match state.current_view {
            View::Dashboard => self.render_dashboard(frame, area, state),
            View::Apps => self.render_apps(frame, area, state),
            View::AppDetail => self.render_app_detail(frame, area, state),
            View::Logs => self.render_logs(frame, area, state),
            View::Settings => self.render_settings(frame, area, state),
        }
    }

    fn render_dashboard(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let total = state.applications.len();
        let running = state.applications.iter().filter(|a| a.status == crate::state::AppStatus::Started).count();
        let stopped = state.applications.iter().filter(|a| a.status == crate::state::AppStatus::Stopped).count();

        let summary = Paragraph::new(format!("Total: {} | Running: {} | Stopped: {}", total, running, stopped))
            .block(Block::bordered(Borders::ALL).title("Overview"));
        frame.render_widget(summary, chunks[0]);

        let actions = List::new([
            ListItem::new("[s] Start | [x] Stop | [r] Restart | [R] Refresh"),
        ])
        .block(Block::bordered(Borders::ALL).title("Quick Actions"));
        frame.render_widget(actions, chunks[1]);
    }

    fn render_apps(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        if state.applications.is_empty() {
            let empty = Paragraph::new("No applications. Press 'R' to refresh.")
                .block(Block::bordered(Borders::ALL).title("Applications"))
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(empty, area);
            return;
        }

        let items: Vec<ListItem> = state.applications.iter().enumerate().map(|(i, app)| {
            let status_color = match app.status {
                crate::state::AppStatus::Started => Color::Green,
                crate::state::AppStatus::Stopped => Color::Red,
                _ => Color::White,
            };
            let line = format!("{:4} | {:20} | {:10} | CPU: {:5.1}% | RAM: {}MB",
                i + 1, app.name, format!("{:?}", app.status), app.cpu_percent, app.memory_mb);
            ListItem::new(line).style(Style::default().fg(status_color))
        }).collect();

        let list = List::new(items)
            .block(Block::bordered(Borders::ALL).title("Applications"))
            .highlight_style(Style::default().bg(Color::DarkGray));
        frame.render_widget(list, area);
    }

    fn render_app_detail(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let app = match state.selected_app() {
            Some(app) => app,
            None => {
                let no_sel = Paragraph::new("No application selected. Go to Applications tab.")
                    .block(Block::bordered(Borders::ALL).title("App Details"))
                    .alignment(ratatui::layout::Alignment::Center);
                frame.render_widget(no_sel, area);
                return;
            }
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let name = Paragraph::new(&app.name).block(Block::bordered(Borders::ALL).title("Name"));
        let status = Paragraph::new(format!("{:?}", app.status))
            .block(Block::bordered(Borders::ALL).title("Status"));
        let resources = Paragraph::new(format!("Workers: {} x {}\nCPU: {:.1}%\nRAM: {} MB",
            app.worker_count, app.worker_type, app.cpu_percent, app.memory_mb))
            .block(Block::bordered(Borders::ALL).title("Resources"));

        frame.render_widget(name, chunks[0]);
        frame.render_widget(status, chunks[1]);
        frame.render_widget(resources, chunks[2]);
    }

    fn render_logs(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let app_name = state.selected_app().map(|a| a.name.clone()).unwrap_or_else(|| "None".to_string());
        let header = Paragraph::new(format!("Logs for: {}", app_name))
            .block(Block::bordered(Borders::ALL).title("Application Logs"));
        frame.render_widget(header, area);
    }

    fn render_settings(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let platform = Paragraph::new(format!("Platform: {}", state.config.anypoint.platform_url))
            .block(Block::bordered(Borders::ALL).title("Anypoint Platform"));
        let env = Paragraph::new(format!("Environment: {}", state.config.anypoint.environment))
            .block(Block::bordered(Borders::ALL).title("Environment"));
        let auth = Paragraph::new(if state.is_authenticated { "✓ Authenticated" } else { "✗ Not authenticated" })
            .block(Block::bordered(Borders::ALL).title("Authentication"));

        frame.render_widget(platform, chunks[0]);
        frame.render_widget(env, chunks[1]);
        frame.render_widget(auth, chunks[2]);
    }

    fn render_status_bar(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let status = if state.is_loading { "Loading..." } else if let Some(ref e) = state.error_message { e.as_str() } else { "Ready" };
        let bar = Paragraph::new(status).style(Style::default().fg(Color::White).bg(Color::Blue));
        frame.render_widget(bar, area);
    }
}