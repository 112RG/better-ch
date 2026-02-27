//! Main TUI application.

use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, List, ListItem, Paragraph, Tabs},
};
use std::io;
use std::panic;

use crate::auth::{Authenticator, User};
use crate::error::Error;
use crate::state::{AppState, SharedState, View};
use crate::ui::views::settings::render_settings;
use tokio::runtime::Runtime;

/// Main TUI application.
pub struct TuiApp {
    pub state: SharedState,
    pub runtime: Runtime,
}

impl TuiApp {
    pub fn new(state: SharedState, runtime: Runtime) -> Self {
        Self { state, runtime }
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Set up panic hook to restore terminal on crash
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            // Try to restore terminal
            let _ = std::io::stdout().execute(LeaveAlternateScreen);
            let _ = disable_raw_mode();
            original_hook(panic_info);
        }));

        // Enable raw mode and alternate screen
        std::io::stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;

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
                        KeyCode::Char('1') => {
                            self.state.blocking_lock().current_view = View::Dashboard
                        }
                        KeyCode::Char('2') => self.state.blocking_lock().current_view = View::Apps,
                        KeyCode::Char('3') => {
                            self.state.blocking_lock().current_view = View::AppDetail
                        }
                        KeyCode::Char('4') => self.state.blocking_lock().current_view = View::Logs,
                        KeyCode::Char('5') => {
                            self.state.blocking_lock().current_view = View::Settings
                        }
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
                        KeyCode::Char('l') | KeyCode::Char('L') => {
                            // Login/Logout - handle in Settings view
                            // Get config first, then release lock
                            let (is_settings, config) = {
                                let s = self.state.blocking_lock();
                                (s.current_view == View::Settings, s.config.clone())
                            };

                            if is_settings {
                                // Get credentials from config
                                let client_id =
                                    config.anypoint.client_id.clone().unwrap_or_default();
                                let client_secret =
                                    config.anypoint.client_secret.clone().unwrap_or_default();

                                if client_id.is_empty() || client_secret.is_empty() {
                                    let mut s = self.state.blocking_lock();
                                    s.error_message = Some("Missing client_id or client_secret. Configure via config file or environment variables.".to_string());
                                    continue;
                                }

                                // Run OAuth login using the runtime
                                let result = self.runtime.block_on(async {
                                    let auth = Authenticator::new(
                                        &config.anypoint.platform_url,
                                        &client_id,
                                        &client_secret,
                                    )?;

                                    // Build and open the authorization URL
                                    let (auth_url, code_verifier, _state) =
                                        auth.build_authorization_url();

                                    println!("Opening browser for login...");
                                    println!("URL: {}", auth_url);

                                    // Open browser
                                    #[cfg(target_os = "windows")]
                                    {
                                        std::process::Command::new("cmd")
                                            .args(["/c", "start", "", &auth_url])
                                            .spawn()
                                            .ok();
                                    }
                                    #[cfg(target_os = "macos")]
                                    {
                                        std::process::Command::new("open")
                                            .arg(&auth_url)
                                            .spawn()
                                            .ok();
                                    }
                                    #[cfg(target_os = "linux")]
                                    {
                                        std::process::Command::new("xdg-open")
                                            .arg(&auth_url)
                                            .spawn()
                                            .ok();
                                    }

                                    // Wait for callback
                                    let code = auth.wait_for_callback()?;

                                    // Exchange code for token
                                    let token =
                                        auth.exchange_code_for_token(&code, &code_verifier).await?;

                                    // Get user info
                                    let user: User = auth.get_current_user(&token).await?;

                                    Ok::<_, Error>((token, user))
                                });

                                match result {
                                    Ok((_token, user)) => {
                                        let mut s = self.state.blocking_lock();
                                        s.is_authenticated = true;
                                        s.error_message =
                                            Some(format!("Logged in as: {}", user.display_name()));
                                    }
                                    Err(e) => {
                                        let mut s = self.state.blocking_lock();
                                        s.error_message = Some(format!("Login failed: {}", e));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Restore terminal on exit
        let _ = std::io::stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();

        Ok(())
    }

    fn render(&self, frame: &mut Frame, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(frame.area());

        self.render_tabs(frame, chunks[0], state);
        self.render_content(frame, chunks[1], state);
        self.render_status_bar(frame, chunks[2], state);
    }

    fn render_tabs(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let titles = vec![
            "Dashboard",
            "Applications",
            "App Details",
            "Logs",
            "Settings",
        ];
        let views = [
            View::Dashboard,
            View::Apps,
            View::AppDetail,
            View::Logs,
            View::Settings,
        ];
        let selected = views
            .iter()
            .position(|v| *v == state.current_view)
            .unwrap_or(0);

        let tabs = Tabs::new(titles)
            .block(Block::bordered().title("CloudHub Runtime Manager"))
            .select(selected)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(tabs, area);
    }

    fn render_content(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        match state.current_view {
            View::Dashboard => self.render_dashboard(frame, area, state),
            View::Apps => self.render_apps(frame, area, state),
            View::AppDetail => self.render_app_detail(frame, area, state),
            View::Logs => self.render_logs(frame, area, state),
            View::Settings => render_settings(frame, area, state),
        }
    }

    fn render_dashboard(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

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

        let summary = Paragraph::new(format!(
            "Total: {} | Running: {} | Stopped: {}",
            total, running, stopped
        ))
        .block(Block::bordered().title("Overview"));
        frame.render_widget(summary, chunks[0]);

        let actions = List::new([ListItem::new(
            "[s] Start | [x] Stop | [r] Restart | [R] Refresh",
        )])
        .block(Block::bordered().title("Quick Actions"));
        frame.render_widget(actions, chunks[1]);
    }

    fn render_apps(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        if state.applications.is_empty() {
            let empty = Paragraph::new("No applications. Press 'R' to refresh.")
                .block(Block::bordered().title("Applications"))
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(empty, area);
            return;
        }

        let items: Vec<ListItem> = state
            .applications
            .iter()
            .enumerate()
            .map(|(i, app)| {
                let status_color = match app.status {
                    crate::state::AppStatus::Started => Color::Green,
                    crate::state::AppStatus::Stopped => Color::Red,
                    _ => Color::White,
                };
                let line = format!(
                    "{:4} | {:20} | {:10} | CPU: {:5.1}% | RAM: {}MB",
                    i + 1,
                    app.name,
                    format!("{:?}", app.status),
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

    fn render_app_detail(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let app = match state.selected_app() {
            Some(app) => app,
            None => {
                let no_sel = Paragraph::new("No application selected. Go to Applications tab.")
                    .block(Block::bordered().title("App Details"))
                    .alignment(ratatui::layout::Alignment::Center);
                frame.render_widget(no_sel, area);
                return;
            }
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        let name = Paragraph::new(app.name.clone()).block(Block::bordered().title("Name"));
        let status =
            Paragraph::new(format!("{:?}", app.status)).block(Block::bordered().title("Status"));
        let resources = Paragraph::new(format!(
            "Workers: {} x {}\nCPU: {:.1}%\nRAM: {} MB",
            app.worker_count, app.worker_type, app.cpu_percent, app.memory_mb
        ))
        .block(Block::bordered().title("Resources"));

        frame.render_widget(name, chunks[0]);
        frame.render_widget(status, chunks[1]);
        frame.render_widget(resources, chunks[2]);
    }

    fn render_logs(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let app_name = state
            .selected_app()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "None".to_string());
        let header = Paragraph::new(format!("Logs for: {}", app_name))
            .block(Block::bordered().title("Application Logs"));
        frame.render_widget(header, area);
    }

    fn render_status_bar(&self, frame: &mut Frame, area: Rect, state: &AppState) {
        let status = if state.is_loading {
            "Loading..."
        } else if let Some(ref e) = state.error_message {
            e.as_str()
        } else {
            "Ready"
        };
        let bar = Paragraph::new(status).style(Style::default().fg(Color::White).bg(Color::Blue));
        frame.render_widget(bar, area);
    }
}
