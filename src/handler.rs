use std::collections::HashMap;
use crate::app::{App, AppResult, Popup, CurrentScreen, MediaType};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use zbus::InterfaceRef;
use crate::dbus::MediaPlayer2Player;
use crate::event::DbusEvent;
use crate::player::mpv::PlayerStatus;


/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(key_event: KeyEvent, app: &mut App, iface_ref: &InterfaceRef<MediaPlayer2Player>) -> AppResult<()> {
    
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
                KeyCode::Char('a') => {
                    app.current_popup = Popup::AddTo;
                    app.set_item_to_be_added(MediaType::Album)?;
                },
                KeyCode::Enter => {
                    app.set_item_to_be_added(MediaType::Album)?;
                    app.add_queue_immediately().await?;
                },
                KeyCode::Tab => {
                    app.cycle_home_pane()?; 
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
                    else {
                        handle_stop_playback(app, iface_ref).await?;
                        app.clear_queue()?;
                    }
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
                },
                KeyCode::Char('A') => {
                    app.current_popup = Popup::AddTo;
                    app.set_item_to_be_added(MediaType::Album)?;
                }
                _ => {}
            }
            Popup::AddTo => match key_event.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.current_popup = Popup::None;
                },
                KeyCode::Char('n') => {
                    app.add_queue_next().await?;
                    app.current_popup = Popup::None;
                },
                KeyCode::Char('e') => {
                    app.add_queue_later().await?;
                    app.current_popup = Popup::None;
                }
                _ => {}
            }
            Popup::None => {}
        }
    }
    
    // Keycodes that should be considered not matter if in popup or not
    if key_event.code == KeyCode::Char('p') { handle_toggle_play_pause(app, iface_ref).await? };
    if key_event.code == KeyCode::Char('r') { handle_shuffle_update(app, iface_ref).await? };
    if key_event.code == KeyCode::Right { handle_seek_forward(app, iface_ref).await? };
    if key_event.code == KeyCode::Left { handle_seek_backwards(app, iface_ref).await? };
    Ok(())
}

pub async fn handle_dbus_events(dbus_event: DbusEvent, app: &mut App, iface_ref: &InterfaceRef<MediaPlayer2Player>) -> AppResult<()> {

    match dbus_event {
        DbusEvent::PlayPause => {
            if *app.player.player_status() == PlayerStatus::Stopped && app.try_play_current() {
                let mut iface = iface_ref.get_mut().await;
                iface.set_position((app.get_playback_time()*1000000) as i64);
                iface.set_playback_status(String::from("Playing"));
                iface.playback_status_changed(iface_ref.signal_context()).await?;
            }
            else {
                handle_toggle_play_pause(app, iface_ref).await?;
            }
        }
        DbusEvent::Next => {app.play_next()?}
        DbusEvent::Previous => {app.play_previous()?}
        DbusEvent::Playing => {
            let mut iface = iface_ref.get_mut().await;
            iface.set_position((app.get_playback_time()*1000000) as i64);
            iface.set_metadata(app.get_metada_for_current_song());
            iface.metadata_changed(iface_ref.signal_context()).await?;
            iface.set_playback_status(String::from("Playing"));
            iface.playback_status_changed(iface_ref.signal_context()).await?;
        }
        DbusEvent::Play => {
            if app.try_play_current() {
                let mut iface = iface_ref.get_mut().await;
                iface.set_position((app.get_playback_time()*1000000) as i64);
                iface.set_playback_status(String::from("Playing"));
                iface.playback_status_changed(iface_ref.signal_context()).await?;
            }
        }
        DbusEvent::Pause => {
            if app.try_pause_current() {
                let mut iface = iface_ref.get_mut().await;
                iface.set_position((app.get_playback_time()*1000000) as i64);
                iface.set_playback_status(String::from("Paused"));
                iface.playback_status_changed(iface_ref.signal_context()).await?;
            }
        }
        DbusEvent::Stop => { handle_stop_playback(app, iface_ref).await?; }
        DbusEvent::SeekForward => handle_seek_forward(app, iface_ref).await?,
        DbusEvent::SeekBackwards => handle_seek_backwards(app, iface_ref).await?,
        DbusEvent::Shuffle => { handle_shuffle_update(app, iface_ref).await?; }
        DbusEvent::Clear => { handle_clear_queue(iface_ref).await?; }
    }
    Ok(())
}

async fn handle_shuffle_update(app: &mut App, iface_ref: &InterfaceRef<MediaPlayer2Player>) -> AppResult<()> {
    let mut iface = iface_ref.get_mut().await;
    iface.set_position((app.get_playback_time()*1000000) as i64);
    
    app.toggle_random_playback()?;
    iface.update_shuffle(app.random_playback);
    iface.shuffle_changed(iface_ref.signal_context()).await?;
    Ok(())
}
async fn handle_seek_forward(app: &mut App, iface_ref: &InterfaceRef<MediaPlayer2Player>) -> AppResult<()> {
    let mut iface = iface_ref.get_mut().await;

    app.player_seek_forward().unwrap();
    let new_position = (app.get_playback_time()*1000000) as i64;
    iface.set_position(new_position);
    MediaPlayer2Player::seeked(iface_ref.signal_context(), new_position).await?;
    
    Ok(())
}
async fn handle_seek_backwards(app: &mut App, iface_ref: &InterfaceRef<MediaPlayer2Player>) -> AppResult<()> {
    let mut iface = iface_ref.get_mut().await;

    app.player_seek_backwards().unwrap();
    let new_position = (app.get_playback_time()*1000000) as i64;
    iface.set_position(new_position);
    MediaPlayer2Player::seeked(iface_ref.signal_context(), new_position).await?;

    Ok(())
}
async fn handle_toggle_play_pause(app: &mut App, iface_ref: &InterfaceRef<MediaPlayer2Player>) -> AppResult<()> {
    app.toggle_playing_status().unwrap();
    let mut iface = iface_ref.get_mut().await;
    iface.set_position((app.get_playback_time()*1000000) as i64);
    if *app.player.player_status() == PlayerStatus::Playing {
        iface.set_playback_status(String::from("Playing"));
    }
    else if *app.player.player_status() == PlayerStatus::Paused {
        iface.set_playback_status(String::from("Paused"));
    }
    iface.playback_status_changed(iface_ref.signal_context()).await?;
    
    Ok(())
}

async fn handle_stop_playback(app: &mut App, iface_ref: &InterfaceRef<MediaPlayer2Player>) -> AppResult<()> {
    
    if *app.player.player_status() != PlayerStatus::Stopped {
        let mut iface = iface_ref.get_mut().await;
        iface.set_position((app.get_playback_time()*1000000) as i64);
        app.stop_playback();
        iface.set_playback_status(String::from("Stopped"));
        iface.playback_status_changed(iface_ref.signal_context()).await?;
    }

    Ok(())
}
async fn handle_clear_queue(iface_ref: &InterfaceRef<MediaPlayer2Player>) -> AppResult<()> {
    let mut iface = iface_ref.get_mut().await;
    iface.set_metadata(HashMap::new());
    iface.metadata_changed(iface_ref.signal_context()).await?;
    Ok(())
}
