use better_ch as app;
use crossterm::event;

fn main() -> std::io::Result<()> {
    let _ = app::config::Config::default();
    let _ = app::error::Error::Auth(app::error::AuthError::TokenExpired);

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| frame.render_widget("Hello World!", frame.area()))?;
            if event::read()?.is_key_press() {
                break Ok(());
            }
        }
    })
}
