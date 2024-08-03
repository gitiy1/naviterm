use crate::app::{App, AppResult, Popup, CurrentScreen, MediaType};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {

    if app.current_popup == Popup::None {
        match app.current_screen {
            CurrentScreen::Home => match key_event.code {
                KeyCode::Char('5') => { app.current_screen = CurrentScreen::Queue; }
                // Exit application on `ESC` or `q`
                KeyCode::Esc | KeyCode::Char('q') => { app.quit(); }
                // Exit application on `Ctrl-C`
                KeyCode::Char('c') | KeyCode::Char('C') => if key_event.modifiers == KeyModifiers::CONTROL { app.quit(); }
                KeyCode::F(1) => { app.current_popup = Popup::ConnectionTest; }
                KeyCode::Char('j') | KeyCode::Down => app.select_next_list()?,
                KeyCode::Char('k') | KeyCode::Up => app.select_previous_list()?,
                KeyCode::Char('i') => {
                    app.get_current_album_information().await?;
                    app.current_popup = Popup::AlbumInformation;
                },
                KeyCode::Enter => {
                    app.set_item_to_be_added(MediaType::Album)?;
                    app.add_queue_immediately().await?;
                }
                // Other handlers you could add here.
                _ => {}
            },
            CurrentScreen::Albums => {}
            CurrentScreen::Playlists => {}
            CurrentScreen::Artists => {}
            CurrentScreen::Queue => match key_event.code {
                KeyCode::Char('1') => {
                    app.current_screen = CurrentScreen::Home;
                }
                KeyCode::Char('l') => { app.play_next()? }
                KeyCode::Char('h') => { app.play_previous()? }
                KeyCode::Char('j') | KeyCode::Down => app.select_next_queue()?,
                KeyCode::Char('k') | KeyCode::Up => app.select_previous_queue()?,
                KeyCode::Char('c') => {
                    if key_event.modifiers == KeyModifiers::CONTROL { app.quit(); }
                    else { app.clear_queue()?; }
                },
                KeyCode::Enter => {
                    app.play_queue_song()?;
                },
                // Exit application on `ESC` or `q`
                KeyCode::Esc | KeyCode::Char('q') => { app.quit() }
                _ => {}
            }
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
                    app.set_item_to_be_added(MediaType::Song)?;
                    app.add_queue_immediately().await?;
                    app.current_popup = Popup::None;
                },
                KeyCode::Char('a') => {
                    app.current_popup = Popup::AddTo;
                    app.set_item_to_be_added(MediaType::Song)?;
                }
                _ => {}
            }
            Popup::AddTo => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.current_popup = Popup::None;
                },
                KeyCode::Char('n') => {
                    app.add_queue_next()?;
                    app.current_popup = Popup::None;
                },
                KeyCode::Char('e') => {
                    app.add_queue_later()?;
                    app.current_popup = Popup::None;
                }
                _ => {}
            }
            Popup::None => {}
        }
    }
    
    // Keycodes that should be considered not matter if in popup or not
    if key_event.code == KeyCode::Char('p') {app.toggle_playing_status()?};
    if key_event.code == KeyCode::Char('r') {app.toggle_random_playback()?};
    if key_event.code == KeyCode::Right {app.player_seek_forward()?};
    if key_event.code == KeyCode::Left {app.player_seek_backwards()?};
    Ok(())
}
