use crate::app::{
    App, AppConnectionMode, AppMovementInList, AppResult, AppStatus, CurrentScreen,
    MediaType, Popup, SortMode, TwoPaneVertical,
};
use crate::constants::VOLUME_STEP;
use crate::dbus::MediaPlayer2Player;
use crate::event::DbusEvent;
use crate::player::mpv::PlayerStatus;
use crossterm::event::{KeyCode, KeyEvent};
use log::{debug, warn};
use std::collections::HashMap;
use zbus::InterfaceRef;
use crate::mappings::ShortcutAction;
use crate::player_data::AppLoopStatus;

/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    iface_ref: Option<&InterfaceRef<MediaPlayer2Player>>,
) -> AppResult<()> {
    let subpane = match app.current_screen {
        CurrentScreen::Albums => app.album_pane.as_str(),
        CurrentScreen::Playlists => app.playlist_pane.as_str(),
        CurrentScreen::Artists => app.artist_pane.as_str(),
        CurrentScreen::Home => app.home_pane.as_str(),
        CurrentScreen::Queue => "any"
    };
    let flag = if app.app_flags.getting_search_string {"searching"}
        else if app.app_flags.is_introducing_new_playlist_name {"introducing_playlist"}
        else if app.app_flags.is_introducing_global_search {"introducing_global"}
        else if app.app_flags.range_year_filter {"range_year"}
        else {"none"};
    let action_parsed = app.shortcuts.get_action_from_shortcut(key_event,app.current_screen.as_str(),subpane, app.current_popup.as_str(), flag);
    debug!("action_parsed {:?}", action_parsed);
    
    match action_parsed {
        ShortcutAction::AddItemEnd => {
            app.add_queue_later()?;
            app.current_popup = Popup::None;
        }
        ShortcutAction::AddItemNext => {
            app.add_queue_next()?;
            app.current_popup = Popup::None;
        }
        ShortcutAction::AddItemPlaylist => app.current_popup = Popup::SelectPlaylist,
        ShortcutAction::CycleLoopMode => {
            match app.player_data.loop_status {
                AppLoopStatus::None => {
                    handle_loop_status_change(app, iface_ref, String::from("Track")).await?;
                }
                AppLoopStatus::Track => {
                    handle_loop_status_change(app, iface_ref, String::from("Playlist")).await?;
                }
                AppLoopStatus::Playlist => {
                    handle_loop_status_change(app, iface_ref, String::from("None")).await?;
                }
            }
        }
        ShortcutAction::CyclePane => {
            app.clear_search()?;
            app.cycle_pane()?;
        }
        ShortcutAction::DeleteItemFromPlaylist => app.delete_selected_song_from_playlist()?,
        ShortcutAction::GoAlbumPane => {
            app.clear_search()?;
            app.current_screen = CurrentScreen::Albums;
        }
        ShortcutAction::GoArtistPane => {
            app.clear_search()?;
            app.current_screen = CurrentScreen::Artists;
        }
        ShortcutAction::GoFirstInList => app.move_in_list(AppMovementInList::First)?,
        ShortcutAction::GoHomePane => {
            app.clear_search()?;
            app.current_screen = CurrentScreen::Home;
        }
        ShortcutAction::GoLastInList => app.move_in_list(AppMovementInList::Last)?,
        ShortcutAction::GoPlaylistPane => {
            app.clear_search()?;
            app.current_screen = CurrentScreen::Playlists;
        }
        ShortcutAction::GoPopupAddAlbumTo => {
            app.current_popup = Popup::AddTo;
            app.set_item_to_be_added(MediaType::Album)?;
        }
        ShortcutAction::GoPopupAddArtistItemTo => {
            app.current_popup = Popup::AddTo;
            app.set_item_to_be_added(app.artist_view_song_or_album())?;
        }
        ShortcutAction::GoPopupAddArtistTo => {
            app.current_popup = Popup::AddTo;
            app.set_item_to_be_added(MediaType::Artist)?;
        }
        ShortcutAction::GoPopupAddPlaylistTo => {
            app.current_popup = Popup::AddTo;
            app.set_item_to_be_added(MediaType::Playlist)?;
        }
        ShortcutAction::GoPopupAddSongTo => {
            app.current_popup = Popup::AddTo;
            app.set_item_to_be_added(MediaType::Song)?;
        }
        ShortcutAction::GoPopupAlbumInfo => app.current_popup = Popup::AlbumInformation,
        ShortcutAction::GoPopupDeletePlaylist => app.current_popup = Popup::ConfirmPlaylistDeletion,
        ShortcutAction::GoPopupGenreFilter => app.current_popup = Popup::GenreFilter,
        ShortcutAction::GoPopupSyncPlaylist => app.current_popup = Popup::SynchronizePlaylist,
        ShortcutAction::GoPopupTestConnection => app.current_popup = Popup::ConnectionTest,
        ShortcutAction::GoPopupUpdateDatabase => {
            app.get_selected_album_for_update()?;
            app.current_popup = Popup::UpdateDatabase;
        }
        ShortcutAction::GoPopupYearFilter => {
            if !app.album_filters.year_to_filter.is_empty() {
                app.app_flags.range_year_filter = true;
            }
            app.current_popup = Popup::YearFilter
        },
        ShortcutAction::GoQueuePane => {
            app.clear_search()?;
            app.current_screen = CurrentScreen::Queue;
        }
        ShortcutAction::GoToTrackAlbum => {
            if !app.player_data.queue.is_empty() {
                app.clear_search()?;
                app.set_album_in_list_to_current_playing()?;
                app.album_pane = TwoPaneVertical::Right;
                app.current_screen = CurrentScreen::Albums;
            }
        }
        ShortcutAction::GoToTrackArtist => {
            if !app.player_data.queue.is_empty() {
                app.clear_search()?;
                app.set_artist_in_list_to_current_playing()?;
                app.artist_pane = TwoPaneVertical::Right;
                app.current_screen = CurrentScreen::Artists;
            }
        }
        ShortcutAction::MoveDownInList => app.move_in_list(AppMovementInList::Next)?,
        ShortcutAction::MovePageDown => app.move_in_list(AppMovementInList::PageDown)?,
        ShortcutAction::MovePageUp => app.move_in_list(AppMovementInList::PageUp)?,
        ShortcutAction::MovePaneDown => {
            app.clear_search()?;
            app.try_go_down_pane()?
        }
        ShortcutAction::MovePaneLeft => {
            app.clear_search()?;
            app.try_go_left_pane()?
        }
        ShortcutAction::MovePaneRight => {
            app.clear_search()?;
            app.try_go_right_pane()?
        },
        ShortcutAction::MovePaneUp => {
            app.clear_search()?;
            app.try_go_up_pane()?
        }
        ShortcutAction::MoveSelectionDown => app.try_move_selection_down()?,
        ShortcutAction::MoveSelectionUp => app.try_move_selection_up()?,
        ShortcutAction::MoveUpInList => app.move_in_list(AppMovementInList::Previous)?,
        ShortcutAction::None => {}
        ShortcutAction::PlayImmediatelyArtist => {
            app.set_item_to_be_added(MediaType::Artist)?;
            app.add_queue_immediately()?;
        }
        ShortcutAction::PlayImmediatelyAlbum => {
            app.set_item_to_be_added(MediaType::Album)?;
            app.add_queue_immediately()?;
        }
        ShortcutAction::PlayImmediatelyPlaylist => {
            app.set_item_to_be_added(MediaType::Playlist)?;
            app.add_queue_immediately()?;
        }
        ShortcutAction::PlayImmediatelySong => {
            app.set_item_to_be_added(MediaType::Song)?;
            app.add_queue_immediately()?;
        }
        ShortcutAction::PlayImmediatelyArtistItem => {
            app.set_item_to_be_added(app.artist_view_song_or_album())?;
            app.add_queue_immediately()?;
        }
        ShortcutAction::PopupClose => {
            app.current_popup = Popup::None;
            app.app_flags.range_year_filter = false;
            app.app_flags.is_introducing_global_search = false;
            app.selected_album_id_to_update.clear();
        }
        ShortcutAction::PopupConfirmDeletionPlaylistNo => app.current_popup = Popup::None,
        ShortcutAction::PopupConfirmDeletionPlaylistYes => {
            app.delete_selected_playlist()?;
            app.current_popup = Popup::None;
        }
        ShortcutAction::PopupConnectionErrorRetry => {
            app.clear_errors_in_operations()?;
            app.status = AppStatus::Updating;
            app.current_popup = Popup::None;
        }
        ShortcutAction::PopupConnectionErrorOffline => {
            app.mode = AppConnectionMode::Offline;
            app.clear_queue()?;
            app.current_popup = Popup::None;
        }
        ShortcutAction::PopupGenreAcceptSelected => {
            app.list_states.album_state.select_first();
            app.set_genre_filter()?;
            app.process_filtered_album_list()?;
            app.current_popup = Popup::None;
            app.clear_search()?;
        }
        ShortcutAction::PopupGenreSelectFavorite => {
            if let KeyCode::Char(c) = key_event.code {
                if c.is_ascii_digit() && c != '0' {
                    let position = c.to_digit(10).unwrap() as usize;
                    if position <= app.database.favorite_genres().len() {
                        app.list_states.album_state.select_first();
                        app.set_favorite_genre_filter(position)?;
                        app.process_filtered_album_list()?;
                        app.current_popup = Popup::None;
                        app.clear_search()?;
                    }
                }
            }
        }
        ShortcutAction::PopupGenreToggleFavorite => app.toggle_favorite_genre()?,
        ShortcutAction::PopupPlaylistAcceptPlaylistName => {
            app.app_flags.is_introducing_new_playlist_name = false;
            app.add_to_playlist()?;
            app.current_popup = Popup::None;
        }
        ShortcutAction::PopupPlaylistAcceptSelected => {
            if app.list_states.popup_select_playlist_list_state.selected().unwrap() == 0
            {
                app.app_flags.is_introducing_new_playlist_name = true;
            } else {
                app.add_to_playlist()?;
                app.current_popup = Popup::None;
            }
        }
        ShortcutAction::PopupPlaylistAddCharToPlaylistName => {
            if let KeyCode::Char(c) = key_event.code {
                app.new_name.push(c);
            }
        }
        ShortcutAction::PopupPlaylistCancelNewPlaylist => {
            app.new_name.clear();
            app.app_flags.is_introducing_new_playlist_name = false;
        }
        ShortcutAction::PopupPlaylistRemoveCharFromPlaylistName => {
            if !app.new_name.is_empty() {
                let mut chars = app.new_name.chars().collect::<Vec<char>>();
                chars.pop();
                app.new_name = chars.iter().collect::<String>();
            }
        }
        ShortcutAction::PopupSynchronizePlaylistPushLocal => {
            if !app.is_selected_playlist_local()? {
                app.push_local_playlist()?;
                app.current_popup = Popup::None;
            }
        }
        ShortcutAction::PopupSynchronizePlaylistPullRemote => {
            if !app.is_selected_playlist_local()? {
                app.pull_remote_playlist()?;
                app.current_popup = Popup::None;
            }
        }
        ShortcutAction::PopupSynchronizeLocalPlaylistPushYes => {
            if app.is_selected_playlist_local()? {
                app.push_local_playlist()?;
                app.current_popup = Popup::None;
            } 
        }
        ShortcutAction::PopupSynchronizeLocalPlaylistPushNo => {
            if app.is_selected_playlist_local()? {
                app.current_popup = Popup::None;
            }
        }
        ShortcutAction::PopupTestConnectionGenerate => app.renew_credentials()?,
        ShortcutAction::PopupTestConnectionTest => app.test_connection().await?,
        ShortcutAction::PopupUpdateDatabaseUpdateAlbums => {
            app.update_alphabetical_albums_async(true)?;
            app.current_popup = Popup::None;
            app.selected_album_id_to_update.clear();
        }
        ShortcutAction::PopupUpdateDatabaseUpdateAllFull => {
            app.populate_db(true)?;
            app.current_popup = Popup::None;
            app.selected_album_id_to_update.clear();
        }
        ShortcutAction::PopupUpdateDatabaseUpdateAllQuick => {
            app.populate_db(false)?;
            app.current_popup = Popup::None;
            app.selected_album_id_to_update.clear();
        }
        ShortcutAction::PopupUpdateDatabaseUpdateCurrentlySelected => {
            if !app.selected_album_id_to_update.is_empty() {
                app.update_selected_album()?;
                app.current_popup = Popup::None;
                app.selected_album_id_to_update.clear();
            }
        }
        ShortcutAction::PopupUpdateDatabaseUpdatePlaylists => {
            app.update_playlists_async(true)?;
            app.current_popup = Popup::None;
            app.selected_album_id_to_update.clear();
        }
        ShortcutAction::PopupYearAcceptFilter => {
            if app.app_flags.range_year_filter {
                app.validate_year_filters()?;
            }
            if app.album_filters.filter_message.is_empty() {
                app.album_filters.year_from_filter =
                    app.album_filters.year_from_filter_new.clone();
                app.album_filters.year_to_filter =
                    app.album_filters.year_to_filter_new.clone();
                app.process_filtered_album_list()?;
                app.app_flags.range_year_filter = false;
                app.current_popup = Popup::None;
            }
        }
        ShortcutAction::PopupYearAddDigit => {
            if let KeyCode::Char(c) = key_event.code {
                if c.is_ascii_digit() {
                    if app.app_flags.is_introducing_to_year {
                        app.album_filters.year_to_filter_new.push(c);
                    } else {
                        app.album_filters.year_from_filter_new.push(c);
                    }
                }
            }
        }
        ShortcutAction::PopupYearRemoveDigit => {
            let input_string = if app.app_flags.is_introducing_to_year {
                &app.album_filters.year_to_filter_new
            } else {
                &app.album_filters.year_from_filter_new
            };
            if !input_string.is_empty() {
                let mut chars = input_string.chars().collect::<Vec<char>>();
                chars.pop();
                if app.app_flags.is_introducing_to_year {
                    app.album_filters.year_to_filter_new = chars.iter().collect::<String>()
                } else {
                    app.album_filters.year_from_filter_new =
                        chars.iter().collect::<String>()
                }
            }

        }
        ShortcutAction::PopupYearToggleFromTo => {
            if app.app_flags.range_year_filter {
                app.app_flags.is_introducing_to_year =
                    !app.app_flags.is_introducing_to_year;
            }
        }
        ShortcutAction::PopupYearToggleRangeInput => {
            if app.app_flags.range_year_filter {
                app.app_flags.range_year_filter = false;
                app.app_flags.is_introducing_to_year = false;
                app.album_filters.year_to_filter.clear();
                app.album_filters.year_to_filter_new.clear();
            } else {
                app.app_flags.range_year_filter = true;
            }
        }
        ShortcutAction::PopupYearClearAndClose => {
            app.album_filters.year_from_filter.clear();
            app.album_filters.year_to_filter.clear();
            app.album_filters.year_from_filter_new.clear();
            app.album_filters.year_to_filter_new.clear();
            app.app_flags.is_introducing_to_year = false;
            app.app_flags.range_year_filter = false;
            app.current_popup = Popup::None;
        }
        ShortcutAction::QueueCenterCursor => app.center_queue_cursor()?,
        ShortcutAction::QueueClear => {
            handle_stop_playback(app, iface_ref).await?;
            app.clear_queue()?;
        }
        ShortcutAction::QueuePlaySong => app.play_queue_song()?,
        ShortcutAction::QuitApp => {
            debug!("Starting app shutdown");
            app.quit();
        }
        ShortcutAction::SearchAccept => {
            app.app_flags.getting_search_string = false;
        }
        ShortcutAction::SearchAddCharToSearchString => {
             if let KeyCode::Char(c) = key_event.code {
                app.search_data.search_string.push(c);
                if app.search_data.search_string.len() > 2 {
                    app.clear_search_results()?;
                    app.search_in_current_list()?;
                    app.go_next_in_search()?;
                }
            }
        }
        ShortcutAction::SearchClear => app.clear_search()?,
        ShortcutAction::SearchEnd => {
            app.app_flags.getting_search_string = false;
            app.clear_search()?;
        }
        ShortcutAction::SearchRemoveCharFromSearchString => {
            if !app.search_data.search_string.is_empty() {
                let mut chars = app.search_data.search_string.chars().collect::<Vec<char>>();
                chars.pop();
                app.search_data.search_string = chars.iter().collect::<String>();
            }
            app.clear_search_results()?;
            if app.search_data.search_string.len() > 2 {
                app.search_in_current_list()?;
                app.go_next_in_search()?;
            }
        }
        ShortcutAction::SearchStart => app.app_flags.getting_search_string = true,
        ShortcutAction::SearchGoNext => app.go_next_in_search()?,
        ShortcutAction::SearchGoPrevious => app.go_previous_in_search()?,
        ShortcutAction::SeekBackwards => handle_seek_backwards(app, iface_ref).await?,
        ShortcutAction::SeekForward => handle_seek_forward(app, iface_ref).await?,
        ShortcutAction::StopPlayback => handle_stop_playback(app, iface_ref).await?,
        ShortcutAction::TogglePlayPause => handle_toggle_play_pause(app, iface_ref).await?,
        ShortcutAction::ToggleRandomPlayback => handle_shuffle_update(app, iface_ref).await?,
        ShortcutAction::ToggleSortMethod => {
            app.album_sorting_mode = if app.album_sorting_mode == SortMode::Alphabetical {
                SortMode::Frequent
            } else {
                SortMode::Alphabetical
            };
            app.clear_search()?;
            app.list_states.album_state.select_first();
            app.process_filtered_album_list()?;
        }
        ShortcutAction::ToggleSortOrder => {
            app.clear_search()?;
            app.toggle_sort_order()?
        }
        ShortcutAction::TrackNext => app.play_next()?,
        ShortcutAction::TrackPrevious => app.play_previous()?,
        ShortcutAction::VolumeDown => {
            let volume = app.get_volume_as_f64()?;
            handle_volume_change(app, iface_ref, volume - VOLUME_STEP).await?;
        }
        ShortcutAction::VolumeUp => {
            let volume = app.get_volume_as_f64()?;
            handle_volume_change(app, iface_ref, volume + VOLUME_STEP).await?;
        }
        ShortcutAction::GoPopupGlobalSearch => {
            if app.search_data.global_search_string.is_empty() || app.current_popup == Popup::GlobalSearch {
                app.app_flags.is_introducing_global_search = true;
            }
            app.current_popup = Popup::GlobalSearch;
        }
        ShortcutAction::PopupGlobalSearchAddCharToSearchString => {
            if let KeyCode::Char(c) = key_event.code {
                app.search_data.global_search_string.push(c);
                if app.search_data.global_search_string.len() > 2 {
                    app.get_global_search_results();
                }
            }
        }
        ShortcutAction::PopupGlobalSearchRemoveCharFromSearchString => {
            if !app.search_data.global_search_string.is_empty() {
                let mut chars = app.search_data.global_search_string.chars().collect::<Vec<char>>();
                chars.pop();
                app.search_data.global_search_string = chars.iter().collect::<String>();
            }
            if app.search_data.global_search_string.len() > 2 {
                app.get_global_search_results();
            }
        }
        ShortcutAction::PopupGlobalSearchAcceptSearchString => {
            app.app_flags.is_introducing_global_search = false;
        }
        ShortcutAction::PopupGlobalSearchClearAndClose => {
            app.current_popup = Popup::None;
            app.app_flags.is_introducing_global_search = false;
            app.search_data.global_search_string.clear();
            app.search_data.global_search_song_results.clear();
        }
        ShortcutAction::PopupGlobalSearchPlayItem => {
            match app.global_search_set_item_to_be_added() {
                Ok(_) => app.add_queue_immediately()?,
                Err(e) => {
                    warn!("Error setting item to be added: {}", e);
                }
            }
            
        }
        ShortcutAction::PopupGlobalSearchAddItemTo => {
            match app.global_search_set_item_to_be_added() {
                Ok(_) => app.current_popup = Popup::AddTo,
                Err(e) => {
                    warn!("Error setting item to be added: {}", e);
                }
            }
            
        }
        ShortcutAction::PopupGlobalSearchGoToAccordingPane => {
            app.go_to_according_pane_for_search_item()?;
            app.current_popup = Popup::None;
        }
    }
    
    Ok(())
}

pub async fn handle_dbus_events(
    dbus_event: DbusEvent,
    app: &mut App,
    iface_ref: Option<&InterfaceRef<MediaPlayer2Player>>,
) -> AppResult<()> {
    match dbus_event {
        DbusEvent::PlayPause => {
            if *app.player.player_status() == PlayerStatus::Stopped && app.try_play_current() {
                if iface_ref.is_none() { return Ok(()) }
                let iface_ref = iface_ref.unwrap();
                let mut iface = iface_ref.get_mut().await;
                iface.update_position((app.get_playback_time() * 1000000) as i64);
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
            if iface_ref.is_none() { return Ok(()) }
            let iface_ref = iface_ref.unwrap();
            let mut iface = iface_ref.get_mut().await;
            iface.update_position((app.get_playback_time() * 1000000) as i64);
            iface.set_playback_status(String::from("Playing"));
            iface
                .playback_status_changed(iface_ref.signal_context())
                .await?;
        }
        DbusEvent::Paused => {
            if iface_ref.is_none() { return Ok(()) }
            let iface_ref = iface_ref.unwrap();
            let mut iface = iface_ref.get_mut().await;
            iface.update_position((app.get_playback_time() * 1000000) as i64);
            iface.set_playback_status(String::from("Paused"));
            iface
                .playback_status_changed(iface_ref.signal_context())
                .await?;
        }
        DbusEvent::Play => {
            if app.try_play_current() {
                if iface_ref.is_none() { return Ok(()) }
                let iface_ref = iface_ref.unwrap();
                let mut iface = iface_ref.get_mut().await;
                iface.update_position((app.get_playback_time() * 1000000) as i64);
                iface.set_playback_status(String::from("Playing"));
                iface
                    .playback_status_changed(iface_ref.signal_context())
                    .await?;
            }
        }
        DbusEvent::Pause => {
            if app.try_pause_current() {
                if iface_ref.is_none() { return Ok(()) }
                let iface_ref = iface_ref.unwrap();
                let mut iface = iface_ref.get_mut().await;
                iface.update_position((app.get_playback_time() * 1000000) as i64);
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
        DbusEvent::LoopStatus(new_loop_status) => {
            handle_loop_status_change(app, iface_ref, new_loop_status).await?;
        }
        DbusEvent::SetPosition(new_position) => {
            handle_position_change(app, iface_ref, new_position).await?;
        }
        DbusEvent::Metadata => {
            if iface_ref.is_none() { return Ok(()) }
            let iface_ref = iface_ref.unwrap();
            let mut iface = iface_ref.get_mut().await;
            iface.update_position((app.get_playback_time() * 1000000) as i64);
            iface.set_metadata(app.get_metadata_for_current_song());
            iface.metadata_changed(iface_ref.signal_context()).await?;
        }
    }
    Ok(())
}

async fn handle_shuffle_update(
    app: &mut App,
    iface_ref: Option<&InterfaceRef<MediaPlayer2Player>>,
) -> AppResult<()> {
    app.toggle_random_playback()?;

    if iface_ref.is_none() { return Ok(()) }
    let iface_ref = iface_ref.unwrap();
    let mut iface = iface_ref.get_mut().await;
    iface.update_position((app.get_playback_time() * 1000000) as i64);
    iface.update_shuffle(app.player_data.random_playback);
    iface.shuffle_changed(iface_ref.signal_context()).await?;
    Ok(())
}
async fn handle_seek_forward(
    app: &mut App,
    iface_ref: Option<&InterfaceRef<MediaPlayer2Player>>,
) -> AppResult<()> {
    app.player_seek_forward()?;

    if iface_ref.is_none() { return Ok(()) }
    let iface_ref = iface_ref.unwrap();
    let mut iface = iface_ref.get_mut().await;
    let new_position = (app.get_playback_time() * 1000000) as i64;
    iface.update_position(new_position);
    MediaPlayer2Player::seeked(iface_ref.signal_context(), new_position).await?;

    Ok(())
}
async fn handle_seek_backwards(
    app: &mut App,
    iface_ref: Option<&InterfaceRef<MediaPlayer2Player>>,
) -> AppResult<()> {
    app.player_seek_backwards()?;

    if iface_ref.is_none() { return Ok(()) }
    let iface_ref = iface_ref.unwrap();
    let mut iface = iface_ref.get_mut().await;
    let new_position = (app.get_playback_time() * 1000000) as i64;
    iface.update_position(new_position);
    MediaPlayer2Player::seeked(iface_ref.signal_context(), new_position).await?;

    Ok(())
}
async fn handle_toggle_play_pause(
    app: &mut App,
    iface_ref: Option<&InterfaceRef<MediaPlayer2Player>>,
) -> AppResult<()> {
    app.toggle_playing_status()?;

    if iface_ref.is_none() { return Ok(()) }
    let iface_ref = iface_ref.unwrap();
    let mut iface = iface_ref.get_mut().await;
    iface.update_position((app.get_playback_time() * 1000000) as i64);
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
    iface_ref: Option<&InterfaceRef<MediaPlayer2Player>>,
) -> AppResult<()> {
    if *app.player.player_status() != PlayerStatus::Stopped {
        app.stop_playback();

        if iface_ref.is_none() { return Ok(()) }
        let iface_ref = iface_ref.unwrap();
        let mut iface = iface_ref.get_mut().await;
        iface.update_position(0i64);
        iface.set_playback_status(String::from("Stopped"));
        iface
            .playback_status_changed(iface_ref.signal_context())
            .await?;
    }

    Ok(())
}
async fn handle_clear_queue(iface_ref: Option<&InterfaceRef<MediaPlayer2Player>>) -> AppResult<()> {
    if iface_ref.is_none() { return Ok(()) }
    let iface_ref = iface_ref.unwrap();
    let mut iface = iface_ref.get_mut().await;
    iface.set_metadata(HashMap::new());
    iface.metadata_changed(iface_ref.signal_context()).await?;
    Ok(())
}
async fn handle_volume_change(
    app: &mut App,
    iface_ref: Option<&InterfaceRef<MediaPlayer2Player>>,
    volume: f64,
) -> AppResult<()> {
    let new_volume = volume.clamp(0.0, 1.0);
    app.set_volume(new_volume)?;

    if iface_ref.is_none() { return Ok(()) }
    let iface_ref = iface_ref.unwrap();
    let mut iface = iface_ref.get_mut().await;
    iface.update_position((app.get_playback_time() * 1000000) as i64);
    iface.update_volume(new_volume);
    iface.volume_changed(iface_ref.signal_context()).await?;
    Ok(())
}

async fn handle_loop_status_change(
    app: &mut App,
    iface_ref: Option<&InterfaceRef<MediaPlayer2Player>>,
    new_loop_status: String,
) -> AppResult<()> {
    app.set_loop_mode(new_loop_status.as_str())?;

    if iface_ref.is_none() { return Ok(()) }
    let iface_ref = iface_ref.unwrap();
    let mut iface = iface_ref.get_mut().await;
    iface.update_position((app.get_playback_time() * 1000000) as i64);
    iface.update_loop_status(new_loop_status);
    iface
        .loop_status_changed(iface_ref.signal_context())
        .await?;
    Ok(())
}

async fn handle_position_change(
    app: &mut App,
    iface_ref: Option<&InterfaceRef<MediaPlayer2Player>>,
    new_position: i64,
) -> AppResult<()> {
    app.set_playback_time(new_position);

    if iface_ref.is_none() { return Ok(()) }
    let iface_ref = iface_ref.unwrap();
    let mut iface = iface_ref.get_mut().await;
    iface.update_position((app.get_playback_time() * 1000000) as i64);
    Ok(())
}
