use crate::app::{App, AppResult, Popup, CurrentScreen};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {

    if app.current_popup == Popup::None {
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
                    app.current_popup = Popup::ConnectionTest;
                }
                KeyCode::Char('j') | KeyCode::Down => app.select_next_list()?,
                KeyCode::Char('k') | KeyCode::Up => app.select_previous_list()?,
                KeyCode::Enter => {
                    app.get_current_album_information().await?;
                    app.current_popup = Popup::AlbumInformation;
                },
                // Other handlers you could add here.
                _ => {}
            }
            _ => {}
        }
    }
    else {
        match app.current_popup {
            Popup::ConnectionTest => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.current_popup = Popup::None;
                }
                KeyCode::Char('r') => {
                    app.renew_credentials()?;
                }
                KeyCode::Char('t') => {
                    app.test_connection().await?;
                }
                _ => {}
            }
            Popup::AlbumInformation => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.current_popup = Popup::None;
                }
                KeyCode::Char('j') | KeyCode::Down => app.select_next_list_popup()?,
                KeyCode::Char('k') | KeyCode::Up => app.select_previous_list_popup()?,
                KeyCode::Enter => {
                    app.add_queue_immediately()?;
                    app.current_popup = Popup::None;
                },
                KeyCode::Char('a') => {
                    app.current_popup = Popup::AddTo;
                    app.set_item_to_be_added()?;
                }
                _ => {}
            }
            Popup::AddTo => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.current_popup = Popup::None;
                }
                _ => {}
            }
            Popup::None => {}
        }
    }
    
    Ok(())
}
