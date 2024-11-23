use crate::app::{App, AppResult, ArtistPane, CurrentScreen, HomePane, MediaType, Popup};
use crate::constants::VOLUME_STEP;
use crate::dbus::MediaPlayer2Player;
use crate::event::DbusEvent;
use crate::player::mpv::PlayerStatus;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::debug;
use std::collections::HashMap;
use zbus::InterfaceRef;

/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    iface_ref: &InterfaceRef<MediaPlayer2Player>,
) -> AppResult<()> {
    if app.getting_search_string {
        match key_event.code {
            KeyCode::Backspace => {
                if !app.search_string.is_empty() {
                    app.search_string.remove(app.search_string.len() - 1);
                }
                app.clear_search_results()?;
                if app.search_string.len() > 2 {
                    app.search_in_current_list()?;
                    app.go_next_in_search()?;
                }
            }
            KeyCode::Enter => {
                app.getting_search_string = false;
            }
            KeyCode::Char(c) => {
                app.search_string.push(c);
                if app.search_string.len() > 2 {
                    app.clear_search_results()?;
                    app.search_in_current_list()?;
                    app.go_next_in_search()?;
                }
            }
            KeyCode::Esc => {
                app.getting_search_string = false;
                app.clear_search()?;
            }
            _ => {}
        }
        return Ok(());
    }
    if app.current_popup == Popup::None {
        match app.current_screen {
            CurrentScreen::Home => match key_event.code {
                KeyCode::Char('2') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Albums;
                }
                KeyCode::Char('3') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Playlists;
                }
                KeyCode::Char('4') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Artists;
                }
                KeyCode::Char('5') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Queue;
                }
                KeyCode::F(1) => {
                    app.current_popup = Popup::ConnectionTest;
                }
                KeyCode::Char('j') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.try_go_down_home_pane()?
                    } else {
                        app.select_next_list()?
                    }
                }
                KeyCode::Char('k') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.try_go_up_home_pane()?
                    } else {
                        app.select_previous_list()?
                    }
                }
                KeyCode::Char('h') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.try_go_left_home_pane()?
                    }
                }
                KeyCode::Char('l') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.try_go_right_home_pane()?
                    }
                }
                KeyCode::Char('i') => {
                    app.current_popup = Popup::AlbumInformation;
                }
                KeyCode::Char('a') => {
                    app.current_popup = Popup::AddTo;
                    app.set_item_to_be_added(MediaType::Album)?;
                }
                KeyCode::Enter => {
                    if app.home_pane == HomePane::BottomRight {
                        app.set_item_to_be_added(MediaType::Song)?;
                    } else {
                        app.set_item_to_be_added(MediaType::Album)?;
                    }
                    app.add_queue_immediately()?;
                }
                KeyCode::Tab => {
                    app.cycle_home_pane()?;
                }
                KeyCode::Char('d') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.page_down()?;
                    }
                }
                KeyCode::Char('u') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.page_up()?;
                    }
                }
                // Other handlers you could add here.
                _ => {}
            },
            CurrentScreen::Albums => match key_event.code {
                KeyCode::Char('1') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Home;
                }
                KeyCode::Char('3') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Playlists;
                }
                KeyCode::Char('4') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Artists;
                }
                KeyCode::Char('5') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Queue;
                }
                KeyCode::Char('j') => app.select_next_list()?,
                KeyCode::Char('k') => app.select_previous_list()?,
                KeyCode::Char('i') => {
                    app.current_popup = Popup::AlbumInformation;
                }
                KeyCode::Char('a') => {
                    app.current_popup = Popup::AddTo;
                    app.set_item_to_be_added(MediaType::Album)?;
                }
                KeyCode::Enter => {
                    app.set_item_to_be_added(MediaType::Album)?;
                    app.add_queue_immediately()?;
                }
                KeyCode::Char('e') => {
                    app.current_popup = Popup::GenreFilter;
                }
                KeyCode::Char('n') => {
                    app.go_next_in_search()?;
                }
                KeyCode::Char('N') => {
                    app.go_previous_in_search()?;
                }
                KeyCode::Char('m') => {
                    app.album_sorting_mode = if app.album_sorting_mode == "alphabetically" {
                        "frequent".to_string()
                    } else {
                        "alphabetically".to_string()
                    };
                    app.clear_search()?;
                    app.list_states.album_state.select_first();
                    app.process_filtered_album_list()?;
                }
                KeyCode::Char('d') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.page_down()?;
                    }
                }
                KeyCode::Char('u') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.page_up()?;
                    }
                }
                _ => {}
            },
            CurrentScreen::Playlists => match key_event.code {
                KeyCode::Char('1') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Home;
                }
                KeyCode::Char('2') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Albums;
                }
                KeyCode::Char('4') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Artists;
                }
                KeyCode::Char('5') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Queue;
                }
                KeyCode::Char('a') => {
                    app.current_popup = Popup::AddTo;
                    app.set_item_to_be_added(MediaType::Playlist)?;
                }
                KeyCode::Enter => {
                    app.set_item_to_be_added(MediaType::Playlist)?;
                    app.add_queue_immediately()?;
                }
                KeyCode::Char('j') => app.select_next_list()?,
                KeyCode::Char('k') => app.select_previous_list()?,
                _ => {}
            },
            CurrentScreen::Artists => match key_event.code {
                KeyCode::Char('1') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Home;
                }
                KeyCode::Char('2') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Albums;
                }
                KeyCode::Char('3') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Playlists;
                }
                KeyCode::Char('5') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Queue;
                }
                KeyCode::Char('j') => app.select_next_list()?,
                KeyCode::Char('k') => app.select_previous_list()?,
                KeyCode::Char('l') => app.artist_pane = ArtistPane::Right,
                KeyCode::Char('h') => app.artist_pane = ArtistPane::Left,
                KeyCode::Char('a') => {
                    app.current_popup = Popup::AddTo;
                    if app.artist_pane == ArtistPane::Left {
                        app.set_item_to_be_added(MediaType::Artist)?;
                    } else {
                        app.set_item_to_be_added(app.artist_view_song_or_album())?;
                    }
                }
                KeyCode::Enter => {
                    if app.artist_pane == ArtistPane::Left {
                        app.set_item_to_be_added(MediaType::Artist)?;
                    } else {
                        app.set_item_to_be_added(app.artist_view_song_or_album())?;
                    }
                    app.add_queue_immediately()?;
                }
                KeyCode::Char('A') => {
                    if app.artist_pane == ArtistPane::Right {
                        app.current_popup = Popup::AddTo;
                        app.set_item_to_be_added(MediaType::Album)?;
                    }
                }
                _ => {}
            },
            CurrentScreen::Queue => match key_event.code {
                KeyCode::Char('1') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Home;
                }
                KeyCode::Char('2') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Albums;
                }
                KeyCode::Char('3') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Playlists;
                }
                KeyCode::Char('4') => {
                    app.clear_search()?;
                    app.current_screen = CurrentScreen::Artists;
                }
                KeyCode::Char('l') => app.play_next()?,
                KeyCode::Char('h') => app.play_previous()?,
                KeyCode::Char('j') => app.select_next_list()?,
                KeyCode::Char('k') => app.select_previous_list()?,
                KeyCode::Char('c') => {
                    if key_event.modifiers != KeyModifiers::CONTROL {
                        handle_stop_playback(app, iface_ref).await?;
                        app.clear_queue()?;
                    }
                }
                KeyCode::Enter => {
                    app.play_queue_song()?;
                }
                KeyCode::Char('d') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.page_down()?;
                    }
                }
                KeyCode::Char('u') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.page_up()?;
                    }
                }
                _ => {}
            },
        }
        // Exit application no matter the current_screen
        // Exit application on `q` or  `<C-c>`
        if key_event.code == KeyCode::Char('q')
            || key_event.code == KeyCode::Char('c') && key_event.modifiers == KeyModifiers::CONTROL
        {
            debug!("Starting app shutdown");
            app.quit();
        } else if key_event.code == KeyCode::Char('/') {
            app.getting_search_string = true;
        }
    } else {
        match app.current_popup {
            Popup::ConnectionTest => match key_event.code {
                KeyCode::Char('r') => {
                    app.renew_credentials()?;
                }
                KeyCode::Char('t') => {
                    app.test_connection().await?;
                }
                _ => {}
            },
            Popup::AlbumInformation => match key_event.code {
                KeyCode::Char('j') => app.select_next_list_popup()?,
                KeyCode::Char('k') => app.select_previous_list_popup()?,
                KeyCode::Enter => {
                    app.set_item_to_be_added(MediaType::Song)?;
                    app.add_queue_immediately()?;
                    app.current_popup = Popup::None;
                }
                KeyCode::Char('a') => {
                    app.current_popup = Popup::AddTo;
                    app.set_item_to_be_added(MediaType::Song)?;
                }
                KeyCode::Char('A') => {
                    app.current_popup = Popup::AddTo;
                    app.set_item_to_be_added(MediaType::Album)?;
                }
                KeyCode::Char('d') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.page_down()?;
                    }
                }
                KeyCode::Char('u') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.page_up()?;
                    }
                }
                _ => {}
            },
            Popup::AddTo => match key_event.code {
                KeyCode::Char('n') => {
                    app.add_queue_next().await?;
                    app.current_popup = Popup::None;
                }
                KeyCode::Char('e') => {
                    app.add_queue_later().await?;
                    app.current_popup = Popup::None;
                }
                _ => {}
            },
            Popup::GenreFilter => match key_event.code {
                KeyCode::Char('j') => app.select_next_list_popup()?,
                KeyCode::Char('k') => app.select_previous_list_popup()?,
                KeyCode::Enter => {
                    app.list_states.album_state.select_first();
                    app.set_genre_filter()?;
                    app.process_filtered_album_list()?;
                    app.current_popup = Popup::None;
                    app.clear_search()?;
                }
                KeyCode::Char('d') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.page_down()?;
                    }
                }
                KeyCode::Char('u') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.page_up()?;
                    }
                }
                _ => {}
            },
            Popup::UpdateDatabase => match key_event.code {
                KeyCode::Char('p') => {
                    app.update_playlists_async(true)?;
                    app.current_popup = Popup::None;
                }
                KeyCode::Char('b') => {
                    app.update_alphabetical_albums_async(true)?;
                    app.current_popup = Popup::None;
                }
                KeyCode::Char('y') => {
                    app.update_playlists_async(true)?;
                    app.current_popup = Popup::None;
                }
                KeyCode::Char('s') => {
                    app.populate_db(false)?;
                    app.current_popup = Popup::None;
                }
                KeyCode::Char('a') => {
                    app.populate_db(true)?;
                    app.current_popup = Popup::None;
                }
                _ => {}
            },
            Popup::None => {}
        }
        // Exit popup no matter the current_popup
        if key_event.code == KeyCode::Esc || key_event.code == KeyCode::Char('q') {
            app.current_popup = Popup::None;
        }
    }

    // Keycodes that should be considered not matter if in popup or not
    if key_event.code == KeyCode::Char('p') || key_event.code == KeyCode::Char(' ') {
        handle_toggle_play_pause(app, iface_ref).await?
    };
    if key_event.code == KeyCode::Char('r') {
        handle_shuffle_update(app, iface_ref).await?
    };
    if key_event.code == KeyCode::Right {
        handle_seek_forward(app, iface_ref).await?
    };
    if key_event.code == KeyCode::Left {
        handle_seek_backwards(app, iface_ref).await?
    };
    if key_event.code == KeyCode::Esc {
        app.clear_search()?;
    };
    if key_event.code == KeyCode::Char('u')
        && key_event.modifiers.is_empty()
        && app.current_popup == Popup::None
    {
        app.current_popup = Popup::UpdateDatabase;
    };
    if key_event.code == KeyCode::Up {
        let volume = app.get_volume_as_f64()?;
        handle_volume_change(app, iface_ref, volume + VOLUME_STEP).await?;
    }
    if key_event.code == KeyCode::Down {
        let volume = app.get_volume_as_f64()?;
        handle_volume_change(app, iface_ref, volume - VOLUME_STEP).await?;
    }
    Ok(())
}

pub async fn handle_dbus_events(
    dbus_event: DbusEvent,
    app: &mut App,
    iface_ref: &InterfaceRef<MediaPlayer2Player>,
) -> AppResult<()> {
    match dbus_event {
        DbusEvent::PlayPause => {
            if *app.player.player_status() == PlayerStatus::Stopped && app.try_play_current() {
                let mut iface = iface_ref.get_mut().await;
                iface.set_position((app.get_playback_time() * 1000000) as i64);
                iface.set_playback_status(String::from("Playing"));
                iface
                    .playback_status_changed(iface_ref.signal_context())
                    .await?;
            } else {
                handle_toggle_play_pause(app, iface_ref).await?;
            }
        }
        DbusEvent::Next => app.play_next()?,
        DbusEvent::Previous => app.play_previous()?,
        DbusEvent::Playing => {
            let mut iface = iface_ref.get_mut().await;
            iface.set_position((app.get_playback_time() * 1000000) as i64);
            iface.set_metadata(app.get_metada_for_current_song());
            iface.metadata_changed(iface_ref.signal_context()).await?;
            iface.set_playback_status(String::from("Playing"));
            iface
                .playback_status_changed(iface_ref.signal_context())
                .await?;
        }
        DbusEvent::Play => {
            if app.try_play_current() {
                let mut iface = iface_ref.get_mut().await;
                iface.set_position((app.get_playback_time() * 1000000) as i64);
                iface.set_playback_status(String::from("Playing"));
                iface
                    .playback_status_changed(iface_ref.signal_context())
                    .await?;
            }
        }
        DbusEvent::Pause => {
            if app.try_pause_current() {
                let mut iface = iface_ref.get_mut().await;
                iface.set_position((app.get_playback_time() * 1000000) as i64);
                iface.set_playback_status(String::from("Paused"));
                iface
                    .playback_status_changed(iface_ref.signal_context())
                    .await?;
            }
        }
        DbusEvent::Stop => {
            handle_stop_playback(app, iface_ref).await?;
        }
        DbusEvent::SeekForward => handle_seek_forward(app, iface_ref).await?,
        DbusEvent::SeekBackwards => handle_seek_backwards(app, iface_ref).await?,
        DbusEvent::Shuffle => {
            handle_shuffle_update(app, iface_ref).await?;
        }
        DbusEvent::Clear => {
            handle_clear_queue(iface_ref).await?;
        }
        DbusEvent::Volume(new_volume) => {
            handle_volume_change(app, iface_ref, new_volume).await?;
        }
    }
    Ok(())
}

async fn handle_shuffle_update(
    app: &mut App,
    iface_ref: &InterfaceRef<MediaPlayer2Player>,
) -> AppResult<()> {
    let mut iface = iface_ref.get_mut().await;
    iface.set_position((app.get_playback_time() * 1000000) as i64);

    app.toggle_random_playback()?;
    iface.update_shuffle(app.random_playback);
    iface.shuffle_changed(iface_ref.signal_context()).await?;
    Ok(())
}
async fn handle_seek_forward(
    app: &mut App,
    iface_ref: &InterfaceRef<MediaPlayer2Player>,
) -> AppResult<()> {
    let mut iface = iface_ref.get_mut().await;

    app.player_seek_forward().unwrap();
    let new_position = (app.get_playback_time() * 1000000) as i64;
    iface.set_position(new_position);
    MediaPlayer2Player::seeked(iface_ref.signal_context(), new_position).await?;

    Ok(())
}
async fn handle_seek_backwards(
    app: &mut App,
    iface_ref: &InterfaceRef<MediaPlayer2Player>,
) -> AppResult<()> {
    let mut iface = iface_ref.get_mut().await;

    app.player_seek_backwards().unwrap();
    let new_position = (app.get_playback_time() * 1000000) as i64;
    iface.set_position(new_position);
    MediaPlayer2Player::seeked(iface_ref.signal_context(), new_position).await?;

    Ok(())
}
async fn handle_toggle_play_pause(
    app: &mut App,
    iface_ref: &InterfaceRef<MediaPlayer2Player>,
) -> AppResult<()> {
    app.toggle_playing_status().unwrap();
    let mut iface = iface_ref.get_mut().await;
    iface.set_position((app.get_playback_time() * 1000000) as i64);
    if *app.player.player_status() == PlayerStatus::Playing {
        iface.set_playback_status(String::from("Playing"));
    } else if *app.player.player_status() == PlayerStatus::Paused {
        iface.set_playback_status(String::from("Paused"));
    }
    iface
        .playback_status_changed(iface_ref.signal_context())
        .await?;

    Ok(())
}

async fn handle_stop_playback(
    app: &mut App,
    iface_ref: &InterfaceRef<MediaPlayer2Player>,
) -> AppResult<()> {
    if *app.player.player_status() != PlayerStatus::Stopped {
        let mut iface = iface_ref.get_mut().await;
        iface.set_position((app.get_playback_time() * 1000000) as i64);
        app.stop_playback();
        iface.set_playback_status(String::from("Stopped"));
        iface
            .playback_status_changed(iface_ref.signal_context())
            .await?;
    }

    Ok(())
}
async fn handle_clear_queue(iface_ref: &InterfaceRef<MediaPlayer2Player>) -> AppResult<()> {
    let mut iface = iface_ref.get_mut().await;
    iface.set_metadata(HashMap::new());
    iface.metadata_changed(iface_ref.signal_context()).await?;
    Ok(())
}
async fn handle_volume_change(
    app: &mut App,
    iface_ref: &InterfaceRef<MediaPlayer2Player>,
    volume: f64,
) -> AppResult<()> {
    let new_volume = volume.clamp(0.0, 1.0);
    app.set_volume(new_volume)?;
    let mut iface = iface_ref.get_mut().await;
    iface.set_position((app.get_playback_time() * 1000000) as i64);
    iface.update_volume(new_volume);
    iface.volume_changed(iface_ref.signal_context()).await?;
    Ok(())
}
