use crate::app::{App, AppResult, CurrentScreen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.current_screen {
        CurrentScreen::Home => match key_event.code {
            // Exit application on `ESC` or `q`
            KeyCode::Esc | KeyCode::Char('q') => {
                app.quit();
            }
            // Exit application on `Ctrl-C`
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.quit();
                }
            }
            KeyCode::F(1) => {
                app.current_screen = CurrentScreen::ConnectionTest;
            }
            // Other handlers you could add here.
            _ => {}
        }
        CurrentScreen::ConnectionTest => match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.current_screen = CurrentScreen::Home;
            }
            KeyCode::Char('r') => {
                app.renew_credentials()?;
            }
            KeyCode::Char('t') => {
                app.test_connection().await?;
            }
            _ => {}
        }
        _ => {}
    }
    Ok(())
}
