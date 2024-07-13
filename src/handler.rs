use crate::app::{App, AppResult, CurrentPopup, CurrentScreen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    
    match app.current_screen {
        CurrentScreen::Home => match key_event.code {
            // Exit application on `ESC` or `q`
            KeyCode::Esc | KeyCode::Char('q') => {
                if app.current_popup == CurrentPopup::None {
                    app.quit();
                }
            }
            // Exit application on `Ctrl-C`
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.quit();
                }
            }
            KeyCode::F(1) => {
                app.current_popup = CurrentPopup::ConnectionTest;
            }
            KeyCode::Char('j') | KeyCode::Down => app.select_next_list()?,
            KeyCode::Char('k') | KeyCode::Up => app.select_previous_list()?,
            // Other handlers you could add here.
            _ => {}
        }
        _ => {}
    }
    
    match app.current_popup {
        CurrentPopup::ConnectionTest => match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.current_popup = CurrentPopup::None;
            }
            KeyCode::Char('r') => {
                app.renew_credentials()?;
            }
            KeyCode::Char('t') => {
                app.test_connection().await?;
            }
            _ => {}
        }
        CurrentPopup::None => {}
    }
    Ok(())
}
