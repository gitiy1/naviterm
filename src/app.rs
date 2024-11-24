use crate::constants;
use crate::event::DbusEvent::{Clear, Playing};
use crate::event::Event;
use crate::event::Event::Dbus;
use crate::model::artist::Artist;
use crate::model::song::Song;
use crate::music_database::MusicDatabase;
use crate::player::ipc::IpcEvent;
use crate::player::mpv::{Mpv, PlayerStatus};
use crate::server::async_operation::Operation;
use crate::server::parser::Parser;
use crate::server::server::Server;
use config::Config;
use log::{debug, info, warn};
use rand::seq::SliceRandom;
use rand::thread_rng;
use ratatui::widgets::ListState;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::error;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

/// Enum with applications screens
#[derive(Debug, PartialEq)]
pub enum CurrentScreen {
    Home,
    Albums,
    Playlists,
    Artists,
    Queue,
}

pub enum AppStatus {
    Connected,
    Disconnected,
    Updating,
}

#[derive(Debug, PartialEq)]
pub enum AppConnectionMode {
    Online,
    Offline,
}

pub enum AppHomeTabMode {
    OneColumn,
    TwoColumns,
}

#[derive(Debug, PartialEq)]
pub enum HomePane {
    Top,
    TopLeft,
    TopRight,
    Bottom,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, PartialEq)]
pub enum ArtistPane {
    Left,
    Right,
}

#[derive(Debug, Default, PartialEq)]
pub enum MediaType {
    #[default]
    Song,
    Album,
    Playlist,
    Artist,
}

/// Enum with applications screens
#[derive(Debug, PartialEq)]
pub enum Popup {
    ConnectionTest,
    AlbumInformation,
    AddTo,
    GenreFilter,
    UpdateDatabase,
    None,
}

/// Application result type.
pub type AppResult<T> = Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
    pub app_flags: AppFlags,
    pub mode: AppConnectionMode,
    pub current_screen: CurrentScreen,
    pub home_pane: HomePane,
    pub home_tab_mode: AppHomeTabMode,
    pub current_popup: Popup,
    pub previous_popup: Popup,
    pub server: Server,
    pub event_sender: Option<UnboundedSender<Event>>,
    pub database: MusicDatabase,
    pub list_states: AppListStates,
    pub item_to_be_added: ItemToBeAdded,
    pub queue: Vec<String>,
    pub queue_order: Vec<usize>,
    pub now_playing: NowPlaying,
    pub player: Mpv,
    pub index_in_queue: usize,
    pub ticks_during_playing_state: usize,
    pub album_genre_filter: String,
    pub album_year_filter: String,
    pub album_sorting_mode: String,
    pub album_sorting_direction: String,
    pub search_string: String,
    pub index_in_search: usize,
    pub search_results_indexes: Vec<usize>,
    pub status: AppStatus,
    pub result_list_alphabetical: Vec<String>,
    pub result_list_most_listened: Vec<String>,
    pub albums_being_updated: usize,
    pub artist_pane: ArtistPane,
}

#[derive(Default, Debug)]
pub struct ItemToBeAdded {
    pub name: String,
    pub id: String,
    pub parent_id: String,
    pub media_type: MediaType,
}

#[derive(Debug, Default, PartialEq)]
pub struct AppFlags {
    pub running: bool,
    pub random_playback: bool,
    pub next_is_in_player_queue: bool,
    pub getting_search_string: bool,
    pub move_to_next_in_search: bool,
    pub upper_case_search: bool,
    pub updating_albums: bool,
    pub updating_alphabetical_albums: bool,
    pub replay_gain_auto: bool,
    pub is_current_song_scrobbled: bool,
}


#[derive(Default)]
pub struct NowPlaying {
    pub id: String,
    pub duration: String,
}

#[derive(Default)]
pub struct AppListStates {
    pub home_tab_top: ListState,
    pub home_tab_top_left: ListState,
    pub home_tab_top_right: ListState,
    pub home_tab_bottom: ListState,
    pub home_tab_bottom_left: ListState,
    pub home_tab_bottom_right: ListState,
    pub queue_list_state: ListState,
    pub popup_list_state: ListState,
    pub popup_genre_list_state: ListState,
    pub album_state: ListState,
    pub playlist_state: ListState,
    pub playlist_selected_state: ListState,
    pub artist_state: ListState,
    pub artist_selected_state: ListState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            app_flags: Default::default(),
            mode: AppConnectionMode::Online,
            current_screen: CurrentScreen::Home,
            home_pane: HomePane::TopLeft,
            home_tab_mode: AppHomeTabMode::TwoColumns,
            current_popup: Popup::None,
            previous_popup: Popup::None,
            server: Server::new(),
            event_sender: None,
            database: MusicDatabase::new(),
            list_states: AppListStates::default(),
            item_to_be_added: ItemToBeAdded::default(),
            queue: vec![],
            queue_order: vec![],
            now_playing: NowPlaying::default(),
            player: Mpv::default(),
            index_in_queue: 0,
            ticks_during_playing_state: 0,
            album_genre_filter: String::from("any"),
            album_year_filter: String::from("any"),
            album_sorting_mode: String::from("alphabetically"),
            album_sorting_direction: String::from("descending"),
            index_in_search: usize::MAX,
            search_string: String::from(""),
            search_results_indexes: vec![],
            status: AppStatus::Connected,
            result_list_most_listened: vec![],
            result_list_alphabetical: vec![],
            albums_being_updated: 0,
            artist_pane: ArtistPane::Left,
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        self.process_pending_requests();
        self.process_player_events();
        if *self.player.player_status() == PlayerStatus::Playing {
            self.ticks_during_playing_state += 1;
            // If we have only 10 seconds left for the current track
            if self.get_playback_time() + 10
                > self.now_playing.duration.as_str().parse::<usize>().unwrap()
            {
                // If there is a next element in queue, add it to mpv queue if it has not been yet added
                if !self.app_flags.next_is_in_player_queue && self.queue_has_next() {
                    let next_index = self.queue_order.get(self.index_in_queue + 1).unwrap();
                    self.player.add_next_song_to_queue(
                        self.server
                            .get_song_url(self.queue.get(*next_index).unwrap().clone())
                            .as_str(),
                    );
                    self.app_flags.next_is_in_player_queue = true;
                }

                if !self.app_flags.is_current_song_scrobbled {
                    // Update last listened album id to remember it for next startup
                    let now_playing_album_id = self
                        .database
                        .get_song(self.now_playing.id.as_str())
                        .album_id()
                        .to_string();
                    self.database
                        .set_last_played_album_id(now_playing_album_id.clone());
                    // The current listened albums is going to be the first in the recent albums
                    // list, but if it is already in the list we remove it first
                    let recent_albums = self.database.recent_albums_mut();
                    let index = recent_albums
                        .iter()
                        .position(|x| x == &now_playing_album_id);
                    match index {
                        None => {}
                        Some(index) => {
                            recent_albums.remove(index);
                        }
                    }
                    recent_albums.insert(0, now_playing_album_id.clone());
                    // Increase playing count in server and locally
                    self.server.scrobble_song_async(self.now_playing.id.clone());
                    self.increase_play_count_for_current_song_in_database(
                        self.now_playing.id.clone(),
                    );
                    self.app_flags.is_current_song_scrobbled = true;
                }
            }
        }
    }

    /// Set running to false in order to quit the application.
    pub fn quit(&mut self) {
        self.player.quit_player();
        self.app_flags.running = false;
    }

    pub fn set_config(&mut self, config: Config) -> AppResult<()> {
        self.server.server_address = config.get("server_address").unwrap();
        self.server.user = config.get("user").unwrap();
        self.server.set_password(config.get("password").unwrap());
        Ok(())
    }

    pub fn renew_credentials(&mut self) -> AppResult<()> {
        self.server.renew_credentials()?;
        Ok(())
    }

    pub async fn test_connection(&mut self) -> AppResult<()> {
        self.server.test_connection().await?;
        Ok(())
    }

    pub fn populate_db(&mut self, force_update: bool) -> AppResult<()> {
        info!(
            "Starting database population. Force update: {}",
            force_update
        );
        self.update_alphabetical_albums_async(force_update)?;
        self.server.get_recent_albums_async();
        self.server.get_recently_added_albums_async();
        self.server.get_most_listened_albums_async(0);
        self.update_playlists_async(force_update)?;
        self.server.get_genres_async();
        Ok(())
    }

    pub fn update_playlists_async(&mut self, force_update: bool) -> AppResult<()> {
        self.server.get_playlists_async(force_update);
        Ok(())
    }

    pub fn update_alphabetical_albums_async(&mut self, force_update: bool) -> AppResult<()> {
        self.server
            .get_album_list_alphabetical_async(force_update, 0);
        Ok(())
    }

    pub fn initialize_player_stream(&mut self) -> AppResult<()> {
        match self.player.initialize() {
            Ok(_) => Ok(()),
            Err(_) => {
                warn!("Could not initialize ipc stream, retrying...");
                sleep(Duration::from_millis(200));
                self.player.initialize()
            }
        }
    }

    pub async fn poll_player_events(&mut self) -> AppResult<()> {
        self.player.poll_ipc_events().await;
        Ok(())
    }

    pub fn select_next_list(&mut self) -> AppResult<()> {
        match self.current_screen {
            CurrentScreen::Home => match self.home_tab_mode {
                AppHomeTabMode::OneColumn => match self.home_pane {
                    HomePane::Top => {
                        self.list_states.home_tab_top.select_next();
                    }
                    HomePane::Bottom => {
                        self.list_states.home_tab_bottom.select_next();
                    }
                    _ => {
                        panic!("Should not reach")
                    }
                },
                AppHomeTabMode::TwoColumns => match self.home_pane {
                    HomePane::TopLeft => {
                        self.list_states.home_tab_top_left.select_next();
                    }
                    HomePane::TopRight => {
                        self.list_states.home_tab_top_right.select_next();
                    }
                    HomePane::BottomLeft => {
                        self.list_states.home_tab_bottom_left.select_next();
                    }
                    HomePane::BottomRight => {
                        self.list_states.home_tab_bottom_right.select_next();
                    }
                    _ => {
                        panic!("Should not reach")
                    }
                },
            },
            CurrentScreen::Albums => {
                self.list_states.album_state.select_next();
            }
            CurrentScreen::Playlists => {
                self.list_states.playlist_state.select_next();
            }
            CurrentScreen::Artists => {
                if self.artist_pane == ArtistPane::Left {
                    self.list_states.artist_state.select_next();
                    self.list_states.artist_selected_state.select_first();
                } else {
                    self.list_states.artist_selected_state.select_next();
                }
            }
            CurrentScreen::Queue => {
                self.list_states.queue_list_state.select_next();
            }
        }

        Ok(())
    }

    pub fn select_previous_list(&mut self) -> AppResult<()> {
        match self.current_screen {
            CurrentScreen::Home => match self.home_pane {
                HomePane::Top => {
                    self.list_states.home_tab_top.select_previous();
                }
                HomePane::Bottom => {
                    self.list_states.home_tab_bottom.select_previous();
                }
                HomePane::TopLeft => {
                    self.list_states.home_tab_top_left.select_previous();
                }
                HomePane::TopRight => {
                    self.list_states.home_tab_top_right.select_previous();
                }
                HomePane::BottomLeft => {
                    self.list_states.home_tab_bottom_left.select_previous();
                }
                HomePane::BottomRight => {
                    self.list_states.home_tab_bottom_right.select_previous();
                }
            },
            CurrentScreen::Albums => self.list_states.album_state.select_previous(),
            CurrentScreen::Playlists => {
                self.list_states.playlist_state.select_previous();
            }
            CurrentScreen::Artists => {
                if self.artist_pane == ArtistPane::Left {
                    self.list_states.artist_state.select_previous();
                    self.list_states.artist_selected_state.select_first();
                } else {
                    self.list_states.artist_selected_state.select_previous();
                }
            }
            CurrentScreen::Queue => {
                self.list_states.queue_list_state.select_previous();
            }
        }
        Ok(())
    }

    pub fn select_next_list_popup(&mut self) -> AppResult<()> {
        match self.current_popup {
            Popup::AlbumInformation => {
                self.list_states.popup_list_state.select_next();
            }
            Popup::GenreFilter => {
                self.list_states.popup_genre_list_state.select_next();
            }
            _ => {
                unreachable!()
            }
        }
        Ok(())
    }

    pub fn select_previous_list_popup(&mut self) -> AppResult<()> {
        match self.current_popup {
            Popup::AlbumInformation => {
                self.list_states.popup_list_state.select_previous();
            }
            Popup::GenreFilter => {
                self.list_states.popup_genre_list_state.select_previous();
            }
            _ => {
                unreachable!()
            }
        }
        Ok(())
    }

    pub fn artist_view_song_or_album(&self) -> MediaType {
        let mut media_type: MediaType = MediaType::Song;
        let albums = self
            .database
            .get_artist(
                self.database
                    .alphabetical_artists()
                    .get(self.list_states.artist_state.selected().unwrap())
                    .unwrap(),
            )
            .albums();
        let mut album_index = 0;
        for album_id in albums {
            let album = self.database.get_album(album_id);
            if album_index == self.list_states.artist_selected_state.selected().unwrap() {
                media_type = MediaType::Album;
                break;
            }
            album_index += album.songs().len() + 1;
            if album_index > self.list_states.artist_selected_state.selected().unwrap() {
                media_type = MediaType::Song;
            }
        }
        media_type
    }

    pub fn add_queue_immediately(&mut self) -> AppResult<()> {
        self.index_in_queue = 0;
        match self.item_to_be_added.media_type {
            MediaType::Song => {
                self.queue.clear();
                self.queue_order.clear();
                self.queue.push(self.item_to_be_added.id.clone());
                self.queue_order.push(self.queue.len() - 1);
                self.change_current_playing_to(self.item_to_be_added.id.clone().as_str());
            }
            MediaType::Album => {
                self.queue.clear();
                self.queue_order.clear();
                let album = self.database.get_album(self.item_to_be_added.id.as_str());
                for song in album.songs() {
                    self.queue.push(song.clone());
                }
                self.queue_order = (0..self.queue.len()).collect();
                if self.app_flags.random_playback {
                    self.shuffle_queue_order_starting_at_current_index()
                }
                self.change_current_playing_to(self.queue.first().unwrap().clone().as_str());
            }
            MediaType::Playlist => {
                self.queue.clear();
                self.queue_order.clear();
                for song_id in self
                    .database
                    .playlists()
                    .get(self.list_states.playlist_state.selected().unwrap())
                    .unwrap()
                    .song_list()
                {
                    self.queue.push(song_id.clone());
                }
                self.queue_order = (0..self.queue.len()).collect();
                if self.app_flags.random_playback {
                    self.shuffle_queue_order_starting_at_current_index()
                }
                self.change_current_playing_to(self.queue.first().unwrap().clone().as_str());
            }
            MediaType::Artist => {
                self.queue.clear();
                self.queue_order.clear();
                let albums = self
                    .database
                    .get_artist(
                        self.database
                            .alphabetical_artists()
                            .get(self.list_states.artist_state.selected().unwrap())
                            .unwrap(),
                    )
                    .albums();
                for album_id in albums {
                    let album = self.database.get_album(album_id.as_str());
                    for song in album.songs() {
                        self.queue.push(song.clone());
                    }
                }
                self.queue_order = (0..self.queue.len()).collect();
                if self.app_flags.random_playback {
                    self.shuffle_queue_order_starting_at_current_index()
                }
                self.change_current_playing_to(self.queue.first().unwrap().clone().as_str());
            }
        }
        self.play_current(false);
        Ok(())
    }

    pub async fn add_queue_next(&mut self) -> AppResult<()> {
        let mut was_empty = false;
        let mut index_to_insert_to = if self.queue.is_empty() {
            was_empty = true;
            0
        } else {
            self.queue
                .iter()
                .position(|x| x == &self.now_playing.id)
                .unwrap()
                + 1
        };
        match self.item_to_be_added.media_type {
            MediaType::Song => {
                self.queue
                    .insert(index_to_insert_to, self.item_to_be_added.id.clone());
                self.queue_order.push(self.queue.len() - 1);
            }
            MediaType::Album => {
                let album = self.database.get_album(self.item_to_be_added.id.as_str());
                for song in album.songs() {
                    self.queue.insert(index_to_insert_to, song.clone());
                    self.queue_order.push(self.queue.len() - 1);
                    index_to_insert_to += 1;
                }
            }
            MediaType::Playlist => {
                for song_id in self
                    .database
                    .playlists()
                    .get(self.list_states.playlist_state.selected().unwrap())
                    .unwrap()
                    .song_list()
                {
                    self.queue.insert(index_to_insert_to, song_id.clone());
                    self.queue_order.push(self.queue.len() - 1);
                    index_to_insert_to += 1;
                }
            }
            MediaType::Artist => {
                for album_id in self
                    .database
                    .get_artist(self.item_to_be_added.id.as_str())
                    .albums()
                {
                    let album = self.database.get_album(album_id.as_str());
                    for song in album.songs() {
                        self.queue.insert(index_to_insert_to, song.clone());
                        self.queue_order.push(self.queue.len() - 1);
                        index_to_insert_to += 1;
                    }
                }
            }
        }
        if was_empty {
            self.change_current_playing_to(self.queue.first().unwrap().clone().as_str());
        }
        Ok(())
    }

    pub async fn add_queue_later(&mut self) -> AppResult<()> {
        let was_empty = self.queue.is_empty();
        match self.item_to_be_added.media_type {
            MediaType::Song => {
                self.queue.push(self.item_to_be_added.id.clone());
                self.queue_order.push(self.queue.len() - 1);
            }
            MediaType::Album => {
                let album = self.database.get_album(self.item_to_be_added.id.as_str());
                for song in album.songs() {
                    self.queue.push(song.clone());
                    self.queue_order.push(self.queue.len() - 1);
                }
            }
            MediaType::Playlist => {
                for song_id in self
                    .database
                    .playlists()
                    .get(self.list_states.playlist_state.selected().unwrap())
                    .unwrap()
                    .song_list()
                {
                    self.queue.push(song_id.clone());
                    self.queue_order.push(self.queue.len() - 1);
                }
            }
            MediaType::Artist => {
                for album_id in self
                    .database
                    .get_artist(self.item_to_be_added.id.as_str())
                    .albums()
                {
                    let album = self.database.get_album(album_id.as_str());
                    for song in album.songs() {
                        self.queue.push(song.clone());
                        self.queue_order.push(self.queue.len() - 1);
                    }
                }
            }
        }
        if was_empty {
            self.change_current_playing_to(self.queue.first().unwrap().clone().as_str());
        }
        Ok(())
    }

    pub fn set_item_to_be_added(&mut self, media: MediaType) -> AppResult<()> {
        let mut selected_album_index;
        let mut offset = 0;
        let album_list = match self.current_screen {
            CurrentScreen::Home => match self.home_tab_mode {
                AppHomeTabMode::OneColumn => match self.home_pane {
                    HomePane::Top => {
                        selected_album_index = self.list_states.home_tab_top.selected().unwrap();
                        self.database.recent_albums()
                    }
                    HomePane::Bottom => {
                        selected_album_index = self.list_states.home_tab_bottom.selected().unwrap();
                        self.database.most_listened_albums()
                    }
                    _ => {
                        panic!("Should not reach")
                    }
                },
                AppHomeTabMode::TwoColumns => match self.home_pane {
                    HomePane::TopLeft => {
                        selected_album_index =
                            self.list_states.home_tab_top_left.selected().unwrap();
                        self.database.recent_albums()
                    }
                    HomePane::TopRight => {
                        selected_album_index =
                            self.list_states.home_tab_top_right.selected().unwrap();
                        self.database.recently_added_albums()
                    }
                    HomePane::BottomLeft => {
                        selected_album_index =
                            self.list_states.home_tab_bottom_left.selected().unwrap();
                        self.database.most_listened_albums()
                    }
                    HomePane::BottomRight => {
                        let album_id_selected = self
                            .database
                            .get_song(
                                self.database
                                    .most_listened_tracks()
                                    .get(self.list_states.home_tab_bottom_right.selected().unwrap())
                                    .unwrap(),
                            )
                            .album_id();
                        selected_album_index = 0;
                        for (i, album_id) in self.database.most_listened_albums().iter().enumerate()
                        {
                            if album_id_selected == album_id {
                                selected_album_index = i;
                            }
                        }
                        self.database.most_listened_albums()
                    }
                    _ => {
                        panic!("Should not reach")
                    }
                },
            },
            CurrentScreen::Albums => {
                selected_album_index = self.list_states.album_state.selected().unwrap();
                self.database.filtered_albums()
            }
            CurrentScreen::Artists => {
                selected_album_index = 0;
                let albums = self
                    .database
                    .get_artist(
                        self.database
                            .alphabetical_artists()
                            .get(self.list_states.artist_state.selected().unwrap())
                            .unwrap(),
                    )
                    .albums();
                // Very hacky way of getting the album index, since the list of the album and songs
                // for the selected artist is not stored anywhere
                for (i, album_id) in albums.iter().enumerate() {
                    let album = self.database.get_album(album_id.as_str());
                    // The list also have the album title as elements, that is way we add 1 more
                    offset += album.songs().len() + 1;
                    if self.list_states.artist_selected_state.selected().unwrap() < offset {
                        selected_album_index = i;
                        // We will need this later
                        offset -= album.songs().len();
                        break;
                    }
                }
                albums
            }
            _ => {
                selected_album_index = 0;
                self.database.filtered_albums()
            }
        };

        match media {
            MediaType::Song => {
                let selected_album_id = album_list.get(selected_album_index).unwrap();
                let songs_ids = self.database.get_album(selected_album_id).songs();
                let song = if self.home_pane == HomePane::BottomRight
                    && self.current_popup == Popup::None
                {
                    self.database.get_song(
                        self.database
                            .most_listened_tracks()
                            .get(self.list_states.home_tab_bottom_right.selected().unwrap())
                            .unwrap(),
                    )
                } else if self.current_screen == CurrentScreen::Artists {
                    self.database.get_song(
                        songs_ids
                            .get(
                                self.list_states.artist_selected_state.selected().unwrap() - offset,
                            )
                            .unwrap(),
                    )
                } else {
                    self.database.get_song(
                        songs_ids
                            .get(self.list_states.popup_list_state.selected().unwrap())
                            .unwrap(),
                    )
                };
                self.item_to_be_added.name = song.title().to_string();
                self.item_to_be_added.id = song.id().to_string();
                self.item_to_be_added.parent_id = selected_album_id.to_string();
                self.item_to_be_added.media_type = MediaType::Song;
            }
            MediaType::Album => {
                self.item_to_be_added.id =
                    album_list.get(selected_album_index).unwrap().to_string();
                self.item_to_be_added.name = self
                    .database
                    .get_album(album_list.get(selected_album_index).unwrap())
                    .name()
                    .to_string();
                self.item_to_be_added.media_type = MediaType::Album;
            }
            MediaType::Playlist => {
                let selected_playlist = self.database.get_playlist(
                    self.database
                        .playlists()
                        .get(self.list_states.playlist_state.selected().unwrap())
                        .unwrap()
                        .id(),
                );
                self.item_to_be_added.name = selected_playlist.name().to_string();
                self.item_to_be_added.id = selected_playlist.id().to_string();
                self.item_to_be_added.media_type = MediaType::Playlist;
            }
            MediaType::Artist => {
                let selected_artist = self.database.get_artist(
                    self.database
                        .alphabetical_artists()
                        .get(self.list_states.artist_state.selected().unwrap())
                        .unwrap(),
                );
                self.item_to_be_added.name = selected_artist.name().to_string();
                self.item_to_be_added.id = selected_artist.id().to_string();
                self.item_to_be_added.media_type = MediaType::Artist;
            }
        }
        Ok(())
    }

    pub fn toggle_playing_status(&mut self) -> AppResult<()> {
        self.player.toggle_play_pause();
        Ok(())
    }

    pub fn toggle_random_playback(&mut self) -> AppResult<()> {
        if self.queue.len() > 1 {
            if self.app_flags.random_playback {
                self.index_in_queue = *self.queue_order.get(self.index_in_queue).unwrap();
                self.queue_order.clear();
                self.queue_order = (0..self.queue.len()).collect();
            } else {
                self.shuffle_queue_order_starting_at_current_index();
                self.index_in_queue = 0;
            }
        }
        if self.app_flags.replay_gain_auto {
            if self.app_flags.random_playback {
                self.player.set_replay_gain("album");
            } else {
                self.player.set_replay_gain("track");
            }
        }
        self.app_flags.random_playback = !self.app_flags.random_playback;
        Ok(())
    }

    pub fn player_seek_forward(&mut self) -> AppResult<()> {
        if self.get_playback_time() + 10
            > self.now_playing.duration.as_str().parse::<usize>().unwrap()
        {
            self.play_next()?;
        } else {
            self.player.seek_forward();
            self.ticks_during_playing_state += 40;
        }
        Ok(())
    }

    pub fn player_seek_backwards(&mut self) -> AppResult<()> {
        self.player.seek_backwards();
        self.ticks_during_playing_state = self.ticks_during_playing_state.saturating_sub(40);
        Ok(())
    }

    pub fn play_next(&mut self) -> AppResult<()> {
        if self.queue_has_next() {
            self.go_next_queue();
            self.play_current(false);
        }
        Ok(())
    }

    pub fn play_previous(&mut self) -> AppResult<()> {
        if self.queue_has_previous() && self.get_playback_time() < 5 {
            self.go_previous_queue();
            self.play_current(false);
        } else {
            self.player.set_playback_percentage("0");
        }
        Ok(())
    }

    fn process_player_events(&mut self) {
        let events = self.player.ipc.events().clone();
        let mut events = events.lock().unwrap();
        while !events.is_empty() {
            match events.pop().unwrap() {
                IpcEvent::FileLoaded => {
                    if self.player.player_status == PlayerStatus::Stopped {
                        self.player.player_status = PlayerStatus::Playing;
                    }
                }
                IpcEvent::PlaybackRestart => {}
                IpcEvent::Eof(reason) => {
                    if reason == "eof" && self.queue_has_next() {
                        self.go_next_queue();
                        self.play_current(true);
                    }
                }
                IpcEvent::Seek => {
                    let playback_time = self.player.get_playback_time();
                    debug!("Got {} as playback time", playback_time);
                    if playback_time != -1.0 {
                        self.ticks_during_playing_state = (playback_time * 4.0).floor() as usize;
                    }
                }
                IpcEvent::Idle => {
                    if self.player.player_status == PlayerStatus::Playing && !self.queue_has_next()
                    {
                        self.player.player_status = PlayerStatus::Stopped;
                        self.event_sender
                            .as_ref()
                            .unwrap()
                            .send(Dbus(Clear))
                            .unwrap();
                    }
                }
                IpcEvent::Error(_) => {}
                IpcEvent::Unrecognized(_) => {}
            }
        }
    }

    pub fn queue_has_next(&self) -> bool {
        if self.queue.len() <= 1 {
            false
        } else {
            self.index_in_queue < self.queue_order.len() - 1
        }
    }

    fn queue_has_previous(&self) -> bool {
        if self.queue.len() <= 1 {
            false
        } else {
            self.index_in_queue > 0
        }
    }

    pub fn get_playback_time(&self) -> usize {
        self.ticks_during_playing_state / 4
    }

    fn go_next_queue(&mut self) {
        self.index_in_queue += 1;
        let next_index = self.queue_order.get(self.index_in_queue).unwrap();
        self.change_current_playing_to(self.queue.get(*next_index).unwrap().clone().as_str());
    }

    fn go_previous_queue(&mut self) {
        self.index_in_queue -= 1;
        let next_index = self.queue_order.get(self.index_in_queue).unwrap();
        self.change_current_playing_to(self.queue.get(*next_index).unwrap().clone().as_str());
    }

    pub fn play_queue_song(&mut self) -> AppResult<()> {
        self.change_current_playing_to(
            self.queue
                .get(self.list_states.queue_list_state.selected().unwrap())
                .unwrap()
                .clone()
                .as_str(),
        );
        debug!(
            "Selected: {}, queue_order: {:?}",
            self.list_states.queue_list_state.selected().unwrap(),
            self.queue_order
        );
        self.index_in_queue = self.list_states.queue_list_state.selected().unwrap();
        if self.app_flags.random_playback {
            self.shuffle_queue_order_starting_at_current_index();
            debug!("queue_order after shuffling: {:?}", self.queue_order);
            self.index_in_queue = 0;
        }
        self.play_current(false);
        Ok(())
    }

    pub fn clear_queue(&mut self) -> AppResult<()> {
        self.queue.clear();
        self.queue_order.clear();
        self.now_playing.id.clear();
        self.index_in_queue = 0;
        self.event_sender
            .as_ref()
            .unwrap()
            .send(Dbus(Clear))
            .unwrap();
        Ok(())
    }

    pub fn try_play_current(&mut self) -> bool {
        if !self.now_playing.id.is_empty() {
            return if self.player.player_status == PlayerStatus::Paused {
                self.toggle_playing_status().unwrap();
                true
            } else if self.player.player_status == PlayerStatus::Stopped {
                self.play_current(false);
                true
            } else {
                false
            };
        }
        false
    }

    pub fn try_pause_current(&mut self) -> bool {
        if !self.now_playing.id.is_empty() && self.player.player_status == PlayerStatus::Playing {
            self.toggle_playing_status().unwrap();
            return true;
        }
        false
    }

    pub fn stop_playback(&mut self) {
        self.player.stop();
        self.player.player_status = PlayerStatus::Stopped;
    }

    fn play_current(&mut self, check_next_song: bool) {
        if check_next_song && self.app_flags.next_is_in_player_queue {
            self.app_flags.next_is_in_player_queue = false;
        } else {
            self.player.play_song(
                self.server
                    .get_song_url(self.now_playing.id.clone())
                    .as_str(),
            );
            self.app_flags.next_is_in_player_queue = false;
        }
        self.event_sender
            .as_ref()
            .unwrap()
            .send(Dbus(Playing))
            .unwrap();
        self.ticks_during_playing_state = 0;
    }

    fn shuffle_queue_order_starting_at_current_index(&mut self) {
        let mut shuffled_vector = Vec::with_capacity(self.queue.len());
        self.queue_order.swap_remove(self.index_in_queue);
        shuffled_vector.push(self.index_in_queue);

        let mut rng = thread_rng();
        self.queue_order.shuffle(&mut rng);

        shuffled_vector.append(&mut self.queue_order);
        self.queue_order = shuffled_vector;
    }

    fn change_current_playing_to(&mut self, new_id: &str) {
        self.now_playing.id = String::from(new_id);
        self.now_playing.duration = String::from(self.database.get_song(new_id).duration());
    }

    pub async fn set_event_handler(&mut self, sender: UnboundedSender<Event>) -> AppResult<()> {
        self.event_sender = Some(sender);
        Ok(())
    }

    pub fn get_metada_for_current_song(&mut self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        let song = self.database.get_song(self.now_playing.id.as_str());
        metadata.insert("title".to_string(), song.title().to_string());
        metadata.insert("album".to_string(), song.album().to_string());
        metadata.insert("artist".to_string(), song.artist().to_string());
        metadata.insert("id".to_string(), song.id().to_string());
        metadata.insert("length".to_string(), song.duration().to_string());
        metadata.insert(
            "cover".to_string(),
            self.server.get_song_art_url(song.id().to_string()),
        );

        metadata
    }

    pub fn cycle_pane(&mut self) -> AppResult<()> {
        match self.current_screen {
            CurrentScreen::Home => match self.home_tab_mode {
                AppHomeTabMode::OneColumn => match self.home_pane {
                    HomePane::Top => {
                        self.home_pane = HomePane::Bottom;
                    }
                    HomePane::Bottom => {
                        self.home_pane = HomePane::Top;
                    }
                    _ => {
                        panic!("Should not reach")
                    }
                },
                AppHomeTabMode::TwoColumns => match self.home_pane {
                    HomePane::TopLeft => {
                        self.home_pane = HomePane::TopRight;
                    }
                    HomePane::TopRight => {
                        self.home_pane = HomePane::BottomLeft;
                    }
                    HomePane::BottomLeft => {
                        self.home_pane = HomePane::BottomRight;
                    }
                    HomePane::BottomRight => {
                        self.home_pane = HomePane::TopLeft;
                    }
                    _ => {
                        panic!("Should not reach")
                    }
                },
            },
            CurrentScreen::Playlists => {}
            CurrentScreen::Artists => {
                if self.artist_pane == ArtistPane::Left { self.artist_pane = ArtistPane::Right }
                else { self.artist_pane = ArtistPane::Left }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn try_go_up_home_pane(&mut self) -> AppResult<()> {
        match self.home_pane {
            HomePane::Bottom => {
                self.home_pane = HomePane::Top;
            }
            HomePane::BottomLeft => {
                self.home_pane = HomePane::TopLeft;
            }
            HomePane::BottomRight => {
                self.home_pane = HomePane::TopRight;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn try_go_down_home_pane(&mut self) -> AppResult<()> {
        match self.home_pane {
            HomePane::Top => {
                self.home_pane = HomePane::Bottom;
            }
            HomePane::TopLeft => {
                self.home_pane = HomePane::BottomLeft;
            }
            HomePane::TopRight => {
                self.home_pane = HomePane::BottomRight;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn try_go_left_home_pane(&mut self) -> AppResult<()> {
        match self.home_pane {
            HomePane::TopRight => {
                self.home_pane = HomePane::TopLeft;
            }
            HomePane::BottomRight => {
                self.home_pane = HomePane::BottomLeft;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn try_go_right_home_pane(&mut self) -> AppResult<()> {
        match self.home_pane {
            HomePane::TopLeft => {
                self.home_pane = HomePane::TopRight;
            }
            HomePane::BottomLeft => {
                self.home_pane = HomePane::BottomRight;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn set_volume(&mut self, new_volume: f64) -> AppResult<()> {
        self.player
            .set_volume((new_volume * 100.0).floor() as usize);
        Ok(())
    }

    pub fn get_volume_as_f64(&mut self) -> AppResult<f64> {
        let volume = self.player.get_volume();
        let volume_as_f64 = volume as f64 / 100.0;
        Ok(volume_as_f64)
    }

    pub fn set_genre_filter(&mut self) -> AppResult<()> {
        self.album_genre_filter =
            if self.list_states.popup_genre_list_state.selected().unwrap() == 0 {
                "any".to_string()
            } else {
                self.database
                    .genres()
                    .get(self.list_states.popup_genre_list_state.selected().unwrap() - 1)
                    .unwrap()
                    .clone()
            };
        Ok(())
    }

    pub fn process_filtered_album_list(&mut self) -> AppResult<()> {
        let mut new_filtered_list: Vec<String> = vec![];
        let list = if self.album_sorting_mode == "frequent" {
            self.database.most_listened_albums()
        } else {
            self.database.alphabetical_list_albums()
        };
        if self.album_genre_filter != "any" {
            for album_id in list {
                if self
                    .database
                    .get_album(album_id)
                    .genres()
                    .contains(&self.album_genre_filter)
                {
                    new_filtered_list.push(album_id.clone());
                }
            }
        } else {
            new_filtered_list = list.clone()
        }
        self.database.set_filtered_albums(new_filtered_list);
        Ok(())
    }

    pub fn page_down(&mut self) -> AppResult<()> {
        if self.current_popup == Popup::None {
            match self.current_screen {
                CurrentScreen::Home => match self.home_tab_mode {
                    AppHomeTabMode::OneColumn => match self.home_pane {
                        HomePane::Top => {
                            self.list_states.home_tab_top.select(Option::from(
                                self.list_states.home_tab_top.selected().unwrap()
                                    + constants::PAGE_SIZE,
                            ));
                        }
                        HomePane::Bottom => {
                            self.list_states.home_tab_bottom.select(Option::from(
                                self.list_states.home_tab_bottom.selected().unwrap()
                                    + constants::PAGE_SIZE,
                            ));
                        }
                        _ => {
                            panic!("Should not reach")
                        }
                    },
                    AppHomeTabMode::TwoColumns => match self.home_pane {
                        HomePane::TopLeft => {
                            self.list_states.home_tab_top_left.select(Option::from(
                                self.list_states.home_tab_top_left.selected().unwrap()
                                    + constants::PAGE_SIZE,
                            ));
                        }
                        HomePane::TopRight => {
                            self.list_states.home_tab_top_right.select(Option::from(
                                self.list_states.home_tab_top_right.selected().unwrap()
                                    + constants::PAGE_SIZE,
                            ));
                        }
                        HomePane::BottomLeft => {
                            self.list_states.home_tab_bottom_left.select(Option::from(
                                self.list_states.home_tab_bottom_left.selected().unwrap()
                                    + constants::PAGE_SIZE,
                            ));
                        }
                        HomePane::BottomRight => {
                            self.list_states.home_tab_bottom_right.select(Option::from(
                                self.list_states.home_tab_bottom_right.selected().unwrap()
                                    + constants::PAGE_SIZE,
                            ));
                        }
                        _ => {
                            panic!("Should not reach")
                        }
                    },
                },
                CurrentScreen::Albums => {
                    self.list_states.album_state.select(Option::from(
                        self.list_states.album_state.selected().unwrap() + constants::PAGE_SIZE,
                    ));
                }
                CurrentScreen::Playlists => {}
                CurrentScreen::Artists => {}
                CurrentScreen::Queue => self.list_states.queue_list_state.select(Option::from(
                    self.list_states.queue_list_state.selected().unwrap() + constants::PAGE_SIZE,
                )),
            }
        } else {
            match self.current_popup {
                Popup::GenreFilter => self.list_states.popup_genre_list_state.select(Option::from(
                    self.list_states.popup_genre_list_state.selected().unwrap()
                        + constants::PAGE_SIZE,
                )),
                Popup::AlbumInformation => self.list_states.popup_list_state.select(Option::from(
                    self.list_states.popup_list_state.selected().unwrap() + constants::PAGE_SIZE,
                )),
                _ => {}
            }
        }
        Ok(())
    }

    pub fn page_up(&mut self) -> AppResult<()> {
        if self.current_popup == Popup::None {
            match self.current_screen {
                CurrentScreen::Home => match self.home_tab_mode {
                    AppHomeTabMode::OneColumn => match self.home_pane {
                        HomePane::Top => {
                            self.list_states.home_tab_top.select(Option::from(
                                self.list_states
                                    .home_tab_top
                                    .selected()
                                    .unwrap()
                                    .saturating_sub(constants::PAGE_SIZE),
                            ));
                        }
                        HomePane::Bottom => {
                            self.list_states.home_tab_bottom.select(Option::from(
                                self.list_states
                                    .home_tab_bottom
                                    .selected()
                                    .unwrap()
                                    .saturating_sub(constants::PAGE_SIZE),
                            ));
                        }
                        _ => {
                            panic!("Should not reach")
                        }
                    },
                    AppHomeTabMode::TwoColumns => match self.home_pane {
                        HomePane::TopLeft => {
                            self.list_states.home_tab_top_left.select(Option::from(
                                self.list_states
                                    .home_tab_top_left
                                    .selected()
                                    .unwrap()
                                    .saturating_sub(constants::PAGE_SIZE),
                            ));
                        }
                        HomePane::TopRight => {
                            self.list_states.home_tab_top_right.select(Option::from(
                                self.list_states
                                    .home_tab_top_right
                                    .selected()
                                    .unwrap()
                                    .saturating_sub(constants::PAGE_SIZE),
                            ));
                        }
                        HomePane::BottomLeft => {
                            self.list_states.home_tab_bottom_left.select(Option::from(
                                self.list_states
                                    .home_tab_bottom_left
                                    .selected()
                                    .unwrap()
                                    .saturating_sub(constants::PAGE_SIZE),
                            ));
                        }
                        HomePane::BottomRight => {
                            self.list_states.home_tab_bottom_right.select(Option::from(
                                self.list_states
                                    .home_tab_bottom_right
                                    .selected()
                                    .unwrap()
                                    .saturating_sub(constants::PAGE_SIZE),
                            ));
                        }
                        _ => {
                            panic!("Should not reach")
                        }
                    },
                },
                CurrentScreen::Albums => {
                    self.list_states.album_state.select(Option::from(
                        self.list_states
                            .album_state
                            .selected()
                            .unwrap()
                            .saturating_sub(constants::PAGE_SIZE),
                    ));
                }
                CurrentScreen::Playlists => {}
                CurrentScreen::Artists => {}
                CurrentScreen::Queue => self.list_states.queue_list_state.select(Option::from(
                    self.list_states
                        .queue_list_state
                        .selected()
                        .unwrap()
                        .saturating_sub(constants::PAGE_SIZE),
                )),
            }
        } else {
            match self.current_popup {
                Popup::GenreFilter => self.list_states.popup_genre_list_state.select(Option::from(
                    self.list_states
                        .popup_genre_list_state
                        .selected()
                        .unwrap()
                        .saturating_sub(constants::PAGE_SIZE),
                )),
                Popup::AlbumInformation => self.list_states.popup_list_state.select(Option::from(
                    self.list_states
                        .popup_list_state
                        .selected()
                        .unwrap()
                        .saturating_sub(constants::PAGE_SIZE),
                )),
                _ => {}
            }
        }
        Ok(())
    }

    pub fn search_in_current_list(&mut self) -> AppResult<()> {
        let list_to_be_searched = match self.current_screen {
            CurrentScreen::Home => match self.home_tab_mode {
                AppHomeTabMode::OneColumn => match self.home_pane {
                    HomePane::Top => self.database.recent_albums(),
                    HomePane::Bottom => self.database.most_listened_albums(),
                    _ => {
                        panic!("Should not reach")
                    }
                },
                AppHomeTabMode::TwoColumns => match self.home_pane {
                    HomePane::TopLeft => self.database.recent_albums(),
                    HomePane::TopRight => self.database.recently_added_albums(),
                    HomePane::BottomLeft => self.database.most_listened_albums(),
                    HomePane::BottomRight => self.database.most_listened_tracks(),
                    _ => {
                        panic!("Should not reach")
                    }
                },
            },
            CurrentScreen::Albums => self.database.filtered_albums(),
            CurrentScreen::Queue => &self.queue,
            _ => return Ok(()),
        };

        self.app_flags.upper_case_search = false;
        for char in self.search_string.chars() {
            self.app_flags.upper_case_search = char.is_uppercase();
            if self.app_flags.upper_case_search {
                break;
            }
        }

        for (index, id) in list_to_be_searched.iter().enumerate() {
            let album = self.database.get_album(id.as_str());
            let album_name_to_search = if self.app_flags.upper_case_search {
                album.name().to_string()
            } else {
                album.name().to_lowercase()
            };
            if album_name_to_search.contains(self.search_string.as_str()) {
                debug!(
                    "album name: {}, matched string: {}",
                    album_name_to_search, self.search_string
                );
                self.search_results_indexes.push(index);
            }
        }

        Ok(())
    }
    pub fn go_next_in_search(&mut self) -> AppResult<()> {
        let list_selected_state = match self.current_screen {
            CurrentScreen::Albums => self.list_states.album_state.selected().unwrap(),
            _ => 0,
        };
        if self.search_results_indexes.is_empty() {
            return Ok(());
        }
        if self.index_in_search == usize::MAX {
            // If we index is equal to MAX, we are starting search
            // We will try to get the search result after the current cursor position
            self.index_in_search = 0;
            while self.search_results_indexes[self.index_in_search] < list_selected_state {
                if self.index_in_search < self.search_results_indexes.len() - 1 {
                    self.index_in_search += 1;
                } else {
                    break;
                }
            }
        } else if self.index_in_search == self.search_results_indexes.len() - 1 {
            // If we are at the end, go back to beginning
            self.index_in_search = 0;
        } else {
            // Else go to next result
            self.index_in_search += 1;
        }
        self.app_flags.move_to_next_in_search = true;
        Ok(())
    }

    pub fn go_previous_in_search(&mut self) -> AppResult<()> {
        if self.search_results_indexes.is_empty() {
            return Ok(());
        }
        if self.index_in_search == usize::MAX || self.index_in_search == 0 {
            self.index_in_search = self.search_results_indexes.len() - 1;
        } else {
            self.index_in_search -= 1;
        }
        self.app_flags.move_to_next_in_search = true;
        Ok(())
    }

    pub fn clear_search(&mut self) -> AppResult<()> {
        self.search_string = "".to_string();
        self.search_results_indexes.clear();
        self.index_in_search = usize::MAX;
        self.app_flags.move_to_next_in_search = false;

        Ok(())
    }

    pub fn clear_search_results(&mut self) -> AppResult<()> {
        self.search_results_indexes.clear();
        self.index_in_search = usize::MAX;

        Ok(())
    }

    pub fn set_replay_gain(&mut self, replay_gain_mode: &str) -> AppResult<()> {
        self.player.set_replay_gain(replay_gain_mode);
        Ok(())
    }

    fn increase_play_count_for_current_song_in_database(&mut self, song_id: String) {
        let song = self.database.get_song_mut(song_id.as_str());
        let play_count = song.play_count().parse::<usize>().unwrap_or_default();
        debug!(
            "Increasing play count for song {} with id {} and play count {}",
            song.title(),
            song.id(),
            play_count
        );
        song.set_play_count((play_count + 1).to_string());
    }

    pub fn process_pending_requests(&mut self) {
        self.server.process_async_operations();

        let pending_operations_number = self.server.operations.len();

        if pending_operations_number > 0 {
            self.status = AppStatus::Updating
        } else {
            self.status = AppStatus::Connected;
        }

        // We will prioritize fetching the alphabetical album list to ensure we have all albums
        // before anything else
        for i in 0..pending_operations_number {
            let operation = &mut self.server.operations[i];
            if operation.finished() && !operation.processed() {
                debug!(
                    "Processing finished operation {:?}, updating_albums: {}, updating_alphabetical_list: {}",
                    operation.operation_id(),
                    self.app_flags.updating_albums,
                    self.app_flags.updating_alphabetical_albums
                );
                match operation.operation_id() {
                    Operation::GetPlaylistList(update) => {
                        if self.app_flags.updating_albums || self.app_flags.updating_alphabetical_albums {
                            continue;
                        }
                        let force_update = *update;
                        let playlist_list =
                            Parser::parse_playlist_list(operation.result().to_string()).unwrap();
                        operation.set_processed(true);
                        for playlist in playlist_list {
                            if self.database.contains_playlist(playlist.id()) && !force_update {
                                debug!("Playlist {} already in database", playlist.name());
                            } else if !self.database.contains_playlist(playlist.id()) {
                                debug!(
                                    "Playlist {} was not in database, fetching",
                                    playlist.name()
                                );
                                self.server.get_playlist_async(playlist.id());
                                self.database
                                    .insert_playlist(playlist.id().to_string(), playlist);
                            } else {
                                debug!("Forcing update for playlist {}", playlist.name());
                                self.server.get_playlist_async(playlist.id());
                            }
                        }
                    }
                    Operation::GetPlaylist(id) => {
                        if self.app_flags.updating_albums || self.app_flags.updating_alphabetical_albums {
                            continue;
                        }
                        self.database.set_playlist_songs(
                            id,
                            Parser::parse_playlist(operation.result().to_string()).unwrap(),
                        );
                        operation.set_processed(true);
                    }
                    Operation::GetAlbumListAlphabetical(update, offset) => {
                        debug!(
                            "Getting alphabetical list. Force update: {}, offset: {}",
                            update, offset
                        );
                        self.app_flags.updating_alphabetical_albums = true;
                        let force_update = *update;
                        let offset = *offset;
                        operation.set_processed(true);
                        let mut album_list =
                            Parser::parse_album_list_simple(operation.result().to_string())
                                .unwrap();
                        if !album_list.is_empty() {
                            let new_offset = offset + album_list.len();
                            for album_id in &album_list {
                                if self.database.contains_album(album_id.as_str()) && !force_update
                                {
                                    debug!("Album {} already in database", album_id);
                                } else if self.database.contains_album(album_id.as_str()) {
                                    debug!("Album {} was not in database, fetching", album_id);
                                    self.albums_being_updated += 1;
                                    self.server.get_album_async(album_id.clone());
                                } else {
                                    debug!("Forcing update for album {}", album_id);
                                    self.albums_being_updated += 1;
                                    self.server.get_album_async(album_id.clone());
                                }
                            }
                            self.result_list_alphabetical.append(&mut album_list);
                            self.server
                                .get_album_list_alphabetical_async(force_update, new_offset);
                        } else {
                            debug!("Alphabetical list was empty, finishing operation");
                            self.database
                                .set_alphabetical_albums(self.result_list_alphabetical.clone());
                            self.app_flags.updating_alphabetical_albums = false;
                            self.result_list_alphabetical.clear();
                            // If there are no albums being updated, it is safe to assume that
                            // we have them all. Else, this flag will be put to false in the
                            // get album operation
                            if self.albums_being_updated == 0 {
                                self.app_flags.updating_albums = false;
                                self.finish_database_update();
                            }
                        }
                    }
                    Operation::GetAlbum(album_id) => {
                        let (album, songs, artist) =
                            Parser::parse_album(operation.result().to_string());
                        for song in songs {
                            if !self.database.contains_song(song.id()) {
                                self.database.insert_song(song.id().to_string(), song);
                            } else {
                                self.database.delete_song(song.id().to_string());
                                self.database.insert_song(song.id().to_string(), song);
                            }
                        }
                        let album_genres = album.genres().clone();
                        if !self.database.contains_album(album_id) {
                            debug!("Album {} not in database, inserting", album.name());
                            self.database.insert_album(album_id.to_string(), album);
                        } else {
                            debug!("Updating album {}", album.name());
                            self.database.delete_album(album_id.to_string());
                            self.database.insert_album(album_id.to_string(), album);
                        }
                        // If we do not have artist in database, create and insert. Otherwise, add
                        // album to it.
                        if !self.database.contains_artist(artist.id()) {
                            debug!("Artist {} not in database, inserting", artist.name());
                            let artist_id = artist.id().to_string();
                            self.database.insert_artist(artist.id().to_string(), artist);
                            self.database
                                .get_artist_mut(artist_id.as_str())
                                .insert_album(album_id.clone(), album_genres);
                        } else {
                            debug!(
                                "Artist {} already in database. Adding album {} to it",
                                artist.name(),
                                album_id
                            );
                            self.database
                                .get_artist_mut(artist.id())
                                .insert_album(album_id.clone(), album_genres);
                        }
                        self.albums_being_updated -= 1;
                        operation.set_processed(true);
                        // If there are no more albums being updated, and we are not updating the
                        // alphabetical list, we can be sure we have them all
                        if self.albums_being_updated == 0 && !self.app_flags.updating_alphabetical_albums {
                            self.finish_database_update();
                            self.app_flags.updating_albums = false;
                        }
                    }
                    Operation::GetAlbumListRecent() => {
                        if self.app_flags.updating_albums || self.app_flags.updating_alphabetical_albums {
                            continue;
                        }
                        operation.set_processed(true);
                        let album_list =
                            Parser::parse_album_list_simple(operation.result().to_string())
                                .unwrap();
                        if !self.database.last_played_album_id().is_empty() {
                            debug!(
                                "Last played album id is {}",
                                self.database.last_played_album_id()
                            );
                            if let Some(last_played_album_index) = album_list
                                .iter()
                                .position(|s| *s == self.database.last_played_album_id())
                            {
                                debug!(
                                    "Found '{}' at index {}/{}",
                                    self.database.last_played_album_id(),
                                    last_played_album_index,
                                    album_list.len()
                                );
                                for album_id in album_list[0..last_played_album_index].iter() {
                                    self.albums_being_updated += 1;
                                    self.server.get_album_async(album_id.clone());
                                }
                                self.database
                                    .set_last_played_album_id(album_list[0].to_string());
                            } else {
                                debug!("Last played album with id '{}' not found in the last played albums!", self.database.last_played_album_id());
                            }
                        }
                        self.database.set_recent_albums(album_list);
                    }
                    Operation::GetAlbumListMostListened(offset) => {
                        if self.app_flags.updating_albums || self.app_flags.updating_alphabetical_albums {
                            continue;
                        }
                        let offset = *offset;
                        operation.set_processed(true);
                        let mut album_list =
                            Parser::parse_album_list_simple(operation.result().to_string())
                                .unwrap();
                        if !album_list.is_empty() {
                            let new_offset = offset + album_list.len();
                            self.result_list_most_listened.append(&mut album_list);
                            self.server.get_most_listened_albums_async(new_offset);
                        } else {
                            debug!("Most listened albums list was empty, finishing operation");
                            self.database
                                .set_most_listened_albums(self.result_list_most_listened.clone());
                            self.result_list_most_listened.clear();
                        }
                    }
                    Operation::GetGenreList => {
                        if self.app_flags.updating_albums || self.app_flags.updating_alphabetical_albums {
                            continue;
                        }
                        let mut genres =
                            Parser::parse_genres_list(operation.result().to_string()).unwrap();
                        genres.sort();
                        self.database.set_genres(genres);
                        operation.set_processed(true);
                    }
                    Operation::GetAlbumListRecentlyAdded() => {
                        if self.app_flags.updating_albums {
                            continue;
                        }
                        let album_list =
                            Parser::parse_album_list_simple(operation.result().to_string())
                                .unwrap();
                        self.database.set_recently_added_albums(album_list);
                        operation.set_processed(true);
                    }
                    Operation::Scrobble(id) => {
                        debug!("Scrobble operation done for song id: {}", id);
                        operation.set_processed(true);
                    }
                }
            }
        }
        self.server.remove_completed_operations();
    }

    fn finish_database_update(&mut self) {
        self.process_filtered_album_list().unwrap();
        self.database
            .set_most_listened_tracks(sort_songs_by_play_count(self.database.songs()));
        self.database
            .set_alphabetical_artists(sort_artists_by_name(self.database.artists()));
    }
}
fn sort_songs_by_play_count(songs: &HashMap<String, Song>) -> Vec<String> {
    let mut song_vector: Vec<_> = songs
        .iter()
        .map(|(id, song)| (id.clone(), song.play_count()))
        .collect();
    song_vector.sort_by(|a, b| {
        b.1.parse::<i32>()
            .unwrap_or(0)
            .cmp(&a.1.parse::<i32>().unwrap_or(0))
    });

    let sorted_ids: Vec<String> = song_vector.into_iter().map(|(id, _)| id).collect();

    sorted_ids
}
fn sort_artists_by_name(artists: &HashMap<String, Artist>) -> Vec<String> {
    let mut artist_vec: Vec<(String, String)> = artists
        .iter()
        .map(|(_, artist)| (artist.name().to_string(), artist.id().to_string()))
        .collect();

    artist_vec.sort_by(|a, b| a.0.cmp(&b.0));

    artist_vec.into_iter().map(|(_, id)| id).collect()
}
