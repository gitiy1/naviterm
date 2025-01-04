use crate::constants;
use crate::event::DbusEvent::{Clear, Playing};
use crate::event::Event;
use crate::event::Event::Dbus;
use crate::model::artist::Artist;
use crate::model::playlist::Playlist;
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

#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
pub enum AppMovementInList {
    Next,
    Previous,
    PageUp,
    PageDown,
    First,
    Last,
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
// Implement a method to cast the enum to a u8
impl HomePane {
    pub fn to_u8(&self) -> u8 {
        match self {
            HomePane::Top => 0,
            HomePane::TopLeft => 1,
            HomePane::TopRight => 2,
            HomePane::Bottom => 3,
            HomePane::BottomLeft => 4,
            HomePane::BottomRight => 5,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TwoPaneVertical {
    Left,
    Right,
}

impl TwoPaneVertical {
    pub fn to_u8(&self) -> u8 {
        match self {
            TwoPaneVertical::Left => 0,
            TwoPaneVertical::Right => 1,
        }
    }
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
    SelectPlaylist,
    SynchronizePlaylist,
    ConfirmPlaylistDeletion,
    None,
    ConnectionError,
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
    pub search_data: SearchData,
    pub status: AppStatus,
    pub result_list_alphabetical: Vec<String>,
    pub result_list_most_listened: Vec<String>,
    pub albums_being_updated: usize,
    pub album_pane: TwoPaneVertical,
    pub artist_pane: TwoPaneVertical,
    pub playlist_pane: TwoPaneVertical,
    pub new_name: String,
    pub queue_data: QueueData,
}

#[derive(Default, Debug)]
pub struct ItemToBeAdded {
    pub name: String,
    pub id: String,
    pub parent_id: String,
    pub media_type: MediaType,
}

pub struct SearchData {
    pub search_string: String,
    pub index_in_search: usize,
    pub search_results_indexes: Vec<usize>,
}

impl Default for SearchData {
    fn default() -> Self {
        SearchData {
            search_string: String::from(""),
            index_in_search: usize::MAX,
            search_results_indexes: Vec::new(),
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct AppFlags {
    pub running: bool,
    pub random_playback: bool,
    pub next_is_in_player_queue: bool,
    pub getting_search_string: bool,
    pub move_to_next_in_search: bool,
    pub upper_case_search: bool,
    pub updating_database: bool,
    pub updating_albums: bool,
    pub updating_alphabetical_albums: bool,
    pub replay_gain_auto: bool,
    pub is_current_song_scrobbled: bool,
    pub is_introducing_new_playlist_name: bool,
}

#[derive(Default)]
pub struct NowPlaying {
    pub id: String,
    pub duration: String,
}

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
    pub popup_select_playlist_list_state: ListState,
    pub album_state: ListState,
    pub album_selected_state: ListState,
    pub playlist_state: ListState,
    pub playlist_selected_state: ListState,
    pub artist_state: ListState,
    pub artist_selected_state: ListState,
}

impl AppListStates {
    fn default() -> AppListStates {
        AppListStates {
            home_tab_top: AppListStates::initialize(),
            home_tab_top_left: AppListStates::initialize(),
            home_tab_top_right: AppListStates::initialize(),
            home_tab_bottom: AppListStates::initialize(),
            home_tab_bottom_left: AppListStates::initialize(),
            home_tab_bottom_right: AppListStates::initialize(),
            queue_list_state: AppListStates::initialize(),
            popup_list_state: AppListStates::initialize(),
            popup_genre_list_state: AppListStates::initialize(),
            popup_select_playlist_list_state: AppListStates::initialize(),
            album_state: AppListStates::initialize(),
            album_selected_state: AppListStates::initialize(),
            playlist_state: AppListStates::initialize(),
            playlist_selected_state: AppListStates::initialize(),
            artist_state: AppListStates::initialize(),
            artist_selected_state: AppListStates::initialize(),
        }
    }

    fn initialize() -> ListState {
        let mut list_state = ListState::default();
        list_state.select_first();
        list_state
    }
}

#[derive(Default)]
pub struct QueueData {
    pub duration_total: String,
    pub duration_left: String,
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
            search_data: SearchData::default(),
            status: AppStatus::Connected,
            result_list_most_listened: vec![],
            result_list_alphabetical: vec![],
            albums_being_updated: 0,
            album_pane: TwoPaneVertical::Left,
            artist_pane: TwoPaneVertical::Left,
            playlist_pane: TwoPaneVertical::Left,
            new_name: String::from(""),
            queue_data: QueueData::default(),
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
        if self.status != AppStatus::Disconnected && self.mode != AppConnectionMode::Offline {
            self.process_pending_requests();
        }
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
                    .get(
                        self.database
                            .alphabetical_playlists()
                            .get(self.list_states.playlist_state.selected().unwrap())
                            .unwrap(),
                    )
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
        self.update_queue_data();
        self.play_current(false);
        Ok(())
    }

    pub fn add_queue_next(&mut self) -> AppResult<()> {
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
                    .get(
                        self.database
                            .alphabetical_playlists()
                            .get(self.list_states.playlist_state.selected().unwrap())
                            .unwrap(),
                    )
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
        self.update_queue_data();
        Ok(())
    }

    pub fn add_queue_later(&mut self) -> AppResult<()> {
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
                    .get(
                        self.database
                            .alphabetical_playlists()
                            .get(self.list_states.playlist_state.selected().unwrap())
                            .unwrap(),
                    )
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
        self.update_queue_data();
        Ok(())
    }

    pub fn add_to_playlist(&mut self) -> AppResult<()> {
        let mut songs_to_add = vec![];
        let mut duration_to_add: usize = 0;
        match self.item_to_be_added.media_type {
            MediaType::Song => {
                songs_to_add.push(self.item_to_be_added.id.clone());
                duration_to_add = self
                    .database
                    .songs()
                    .get(&self.item_to_be_added.id)
                    .unwrap()
                    .duration()
                    .parse()
                    .unwrap();
            }
            MediaType::Album => {
                let album = self.database.get_album(self.item_to_be_added.id.as_str());
                for song in album.songs() {
                    songs_to_add.push(song.clone());
                    duration_to_add += self
                        .database
                        .songs()
                        .get(song)
                        .unwrap()
                        .duration()
                        .parse::<usize>()
                        .unwrap();
                }
            }
            MediaType::Playlist => {
                let playlist = self
                    .database
                    .get_playlist(self.item_to_be_added.id.as_str());
                for song in playlist.song_list() {
                    songs_to_add.push(song.clone());
                    duration_to_add += self
                        .database
                        .songs()
                        .get(song)
                        .unwrap()
                        .duration()
                        .parse::<usize>()
                        .unwrap();
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
                        songs_to_add.push(song.clone());
                        duration_to_add += self
                            .database
                            .songs()
                            .get(song)
                            .unwrap()
                            .duration()
                            .parse::<usize>()
                            .unwrap();
                    }
                }
            }
        }
        let mut index = self
            .list_states
            .popup_select_playlist_list_state
            .selected()
            .unwrap();
        if index == 0 {
            let mut new_playlist = Playlist::default();
            self.database
                .set_number_of_local_playlists(self.database.number_of_local_playlists() + 1);
            let playlist_id = "local_".to_owned()
                + self
                    .database
                    .number_of_local_playlists()
                    .to_string()
                    .as_str();
            new_playlist.set_id(playlist_id.clone());
            new_playlist.set_name(self.new_name.clone());
            new_playlist.set_song_count(songs_to_add.len().to_string());
            new_playlist.set_duration(duration_to_add.to_string());
            new_playlist.song_list_mut().append(&mut songs_to_add);
            self.database.insert_playlist(playlist_id, new_playlist);
            self.database
                .set_alphabetical_playlists(sort_playlists_by_name(self.database.playlists()));
            self.new_name.clear();
        } else {
            index -= 1;
            let playlist_id = self
                .database
                .alphabetical_playlists()
                .get(index)
                .unwrap()
                .clone();
            let playlist = self.database.get_mut_playlist(playlist_id.as_str());
            let duration = playlist.duration().parse::<usize>().unwrap();
            let song_count = playlist.song_count().parse::<usize>().unwrap();
            playlist.song_list_mut().append(&mut songs_to_add);
            playlist.set_duration((duration + duration_to_add).to_string());
            playlist.set_song_count((song_count + songs_to_add.len()).to_string());
            if !playlist.id().starts_with("local") {
                playlist.set_modified(true);
            }
        }
        self.update_queue_data();
        Ok(())
    }

    fn update_queue_data(&mut self) {
        let mut duration_total = 0;
        let mut duration_left = 0;

        // Walk the queue and add songs duration to the total duration field. If the loop index is
        // greater than the index in queue, we add that song duration to the remaining duration as
        // well.
        for (i, index_in_order_queue) in self.queue_order.iter().enumerate() {
            let song = self
                .database
                .get_song(self.queue.get(*index_in_order_queue).unwrap());
            duration_total += song.duration().parse::<usize>().unwrap();
            if i >= self.index_in_queue {
                duration_left += song.duration().parse::<usize>().unwrap();
            }
        }

        self.queue_data.duration_total = duration_total.to_string();
        self.queue_data.duration_left = duration_left.to_string();
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
                } else if self.current_screen == CurrentScreen::Albums {
                    self.database.get_song(
                        songs_ids
                            .get(self.list_states.album_selected_state.selected().unwrap())
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
                } else if self.current_screen == CurrentScreen::Playlists {
                    self.database.get_song(
                        self.database
                            .playlists()
                            .get(
                                self.database
                                    .alphabetical_playlists()
                                    .get(self.list_states.playlist_state.selected().unwrap())
                                    .unwrap(),
                            )
                            .unwrap()
                            .song_list()
                            .get(self.list_states.playlist_selected_state.selected().unwrap())
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
                        .get(
                            self.database
                                .alphabetical_playlists()
                                .get(self.list_states.playlist_state.selected().unwrap())
                                .unwrap(),
                        )
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

    pub fn push_local_playlist(&mut self) -> AppResult<()> {
        let playlist = self
            .database
            .playlists()
            .get(
                self.database
                    .alphabetical_playlists()
                    .get(self.list_states.playlist_state.selected().unwrap())
                    .unwrap(),
            )
            .unwrap();
        self.server
            .update_playlist_async(playlist.song_list().clone(), playlist.id());
        Ok(())
    }

    pub fn pull_remote_playlist(&mut self) -> AppResult<()> {
        let playlist = self
            .database
            .playlists()
            .get(
                self.database
                    .alphabetical_playlists()
                    .get(self.list_states.playlist_state.selected().unwrap())
                    .unwrap(),
            )
            .unwrap();
        self.server.get_playlist_async(playlist.id());
        Ok(())
    }

    pub fn delete_selected_song_from_playlist(&mut self) -> AppResult<()> {
        let playlist_id = self
            .database
            .alphabetical_playlists()
            .get(self.list_states.playlist_state.selected().unwrap())
            .unwrap()
            .clone();
        let song_index = self.list_states.playlist_selected_state.selected().unwrap();
        let playlist = self.database.get_mut_playlist(playlist_id.as_str());
        playlist.song_list_mut().remove(song_index);
        playlist.set_modified(true);

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
        self.update_queue_data();
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
        self.update_queue_data();
    }

    fn go_previous_queue(&mut self) {
        self.index_in_queue -= 1;
        let next_index = self.queue_order.get(self.index_in_queue).unwrap();
        self.change_current_playing_to(self.queue.get(*next_index).unwrap().clone().as_str());
        self.update_queue_data();
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
        self.update_queue_data();
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
            self.app_flags.is_current_song_scrobbled = false;
        } else {
            self.player.play_song(
                self.server
                    .get_song_url(self.now_playing.id.clone())
                    .as_str(),
            );
            self.app_flags.next_is_in_player_queue = false;
            self.app_flags.is_current_song_scrobbled = false;
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
            CurrentScreen::Albums => {
                if self.album_pane == TwoPaneVertical::Left {
                    self.album_pane = TwoPaneVertical::Right
                } else {
                    self.album_pane = TwoPaneVertical::Left
                }
            }
            CurrentScreen::Playlists => {
                if self.playlist_pane == TwoPaneVertical::Left {
                    self.playlist_pane = TwoPaneVertical::Right
                } else {
                    self.playlist_pane = TwoPaneVertical::Left
                }
            }
            CurrentScreen::Artists => {
                if self.artist_pane == TwoPaneVertical::Left {
                    self.artist_pane = TwoPaneVertical::Right
                } else {
                    self.artist_pane = TwoPaneVertical::Left
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn try_go_up_pane(&mut self) -> AppResult<()> {
        match self.current_screen {
            CurrentScreen::Home => match self.home_pane {
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
            },
            _ => {}
        }
        Ok(())
    }

    pub fn try_go_down_pane(&mut self) -> AppResult<()> {
        match self.current_screen {
            CurrentScreen::Home => match self.home_pane {
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
            },
            _ => {}
        }
        Ok(())
    }

    pub fn try_go_left_pane(&mut self) -> AppResult<()> {
        match self.current_screen {
            CurrentScreen::Home => match self.home_pane {
                HomePane::TopRight => {
                    self.home_pane = HomePane::TopLeft;
                }
                HomePane::BottomRight => {
                    self.home_pane = HomePane::BottomLeft;
                }
                _ => {}
            },
            CurrentScreen::Playlists => {
                self.playlist_pane = TwoPaneVertical::Left;
            }
            CurrentScreen::Artists => {
                self.artist_pane = TwoPaneVertical::Left;
            }
            _ => {}
        }

        Ok(())
    }
    pub fn try_go_right_pane(&mut self) -> AppResult<()> {
        match self.current_screen {
            CurrentScreen::Home => match self.home_pane {
                HomePane::TopLeft => {
                    self.home_pane = HomePane::TopRight;
                }
                HomePane::BottomLeft => {
                    self.home_pane = HomePane::BottomRight;
                }
                _ => {}
            },
            CurrentScreen::Playlists => {
                self.playlist_pane = TwoPaneVertical::Right;
            }
            CurrentScreen::Artists => {
                self.artist_pane = TwoPaneVertical::Right;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn try_move_selection_down(&mut self) -> AppResult<()> {
        let current_index = self.list_states.playlist_selected_state.selected().unwrap();
        let playlist_id = self
            .database
            .alphabetical_playlists()
            .get(self.list_states.playlist_state.selected().unwrap())
            .unwrap()
            .clone();
        let playlist = self.database.get_mut_playlist(playlist_id.as_str());
        let max_index = playlist.song_list_mut().len();
        if current_index < max_index - 1 {
            playlist
                .song_list_mut()
                .swap(current_index, current_index + 1);
            playlist.set_modified(true);
            self.list_states
                .playlist_selected_state
                .select(Some(current_index + 1));
        }
        Ok(())
    }

    pub fn try_move_selection_up(&mut self) -> AppResult<()> {
        let current_index = self.list_states.playlist_selected_state.selected().unwrap();
        let playlist_id = self
            .database
            .alphabetical_playlists()
            .get(self.list_states.playlist_state.selected().unwrap())
            .unwrap()
            .clone();
        let playlist = self.database.get_mut_playlist(playlist_id.as_str());
        if current_index > 0 {
            playlist
                .song_list_mut()
                .swap(current_index, current_index - 1);
            playlist.set_modified(true);
            self.list_states
                .playlist_selected_state
                .select(Some(current_index - 1));
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

    pub fn center_queue_cursor(&mut self) -> AppResult<()> {
        self.list_states
            .queue_list_state
            .select(Some(self.queue_order[self.index_in_queue]));
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

    pub fn move_in_list(&mut self, move_operation: AppMovementInList) -> AppResult<()> {
        let list_state_item = if self.current_popup == Popup::None {
            match self.current_screen {
                CurrentScreen::Home => match self.home_pane {
                    HomePane::Top => &mut self.list_states.home_tab_top,
                    HomePane::TopLeft => &mut self.list_states.home_tab_top_left,
                    HomePane::TopRight => &mut self.list_states.home_tab_top_right,
                    HomePane::Bottom => &mut self.list_states.home_tab_bottom,
                    HomePane::BottomLeft => &mut self.list_states.home_tab_bottom_left,
                    HomePane::BottomRight => &mut self.list_states.home_tab_bottom_right,
                },
                CurrentScreen::Albums => match self.album_pane {
                    TwoPaneVertical::Left => &mut self.list_states.album_state,
                    TwoPaneVertical::Right => &mut self.list_states.album_selected_state,
                },
                CurrentScreen::Playlists => {
                    if self.playlist_pane == TwoPaneVertical::Left {
                        &mut self.list_states.playlist_state
                    } else {
                        &mut self.list_states.playlist_selected_state
                    }
                }
                CurrentScreen::Artists => {
                    if self.artist_pane == TwoPaneVertical::Left {
                        &mut self.list_states.artist_state
                    } else {
                        &mut self.list_states.artist_selected_state
                    }
                }
                CurrentScreen::Queue => &mut self.list_states.queue_list_state,
            }
        } else {
            match self.current_popup {
                Popup::AlbumInformation => &mut self.list_states.popup_list_state,
                Popup::GenreFilter => &mut self.list_states.popup_genre_list_state,
                Popup::SelectPlaylist => &mut self.list_states.popup_select_playlist_list_state,
                _ => &mut self.list_states.popup_list_state,
            }
        };
        match move_operation {
            AppMovementInList::Next => list_state_item.select_next(),
            AppMovementInList::Previous => list_state_item.select_previous(),
            AppMovementInList::PageUp => {
                list_state_item.select(Option::from(
                    list_state_item
                        .selected()
                        .unwrap()
                        .saturating_sub(constants::PAGE_SIZE),
                ));
            }
            AppMovementInList::PageDown => {
                list_state_item.select(Option::from(
                    list_state_item.selected().unwrap() + constants::PAGE_SIZE,
                ));
            }
            AppMovementInList::First => list_state_item.select_first(),
            AppMovementInList::Last => list_state_item.select_last(),
        }
        Ok(())
    }

    pub fn search_in_current_list(&mut self) -> AppResult<()> {
        let list_to_be_searched = match self.current_screen {
            CurrentScreen::Home => match self.home_tab_mode {
                AppHomeTabMode::OneColumn => match self.home_pane {
                    HomePane::Top => self
                        .database
                        .recent_albums()
                        .iter()
                        .map(|album_id| self.database.get_album(album_id).name().to_string())
                        .collect::<Vec<String>>(),
                    HomePane::Bottom => self
                        .database
                        .most_listened_albums()
                        .iter()
                        .map(|album_id| self.database.get_album(album_id).name().to_string())
                        .collect::<Vec<String>>(),
                    _ => {
                        panic!("Should not reach")
                    }
                },
                AppHomeTabMode::TwoColumns => match self.home_pane {
                    HomePane::TopLeft => self
                        .database
                        .recent_albums()
                        .iter()
                        .map(|album_id| self.database.get_album(album_id).name().to_string())
                        .collect::<Vec<String>>(),
                    HomePane::TopRight => self
                        .database
                        .recently_added_albums()
                        .iter()
                        .map(|album_id| self.database.get_album(album_id).name().to_string())
                        .collect::<Vec<String>>(),
                    HomePane::BottomLeft => self
                        .database
                        .most_listened_albums()
                        .iter()
                        .map(|album_id| self.database.get_album(album_id).name().to_string())
                        .collect::<Vec<String>>(),
                    HomePane::BottomRight => self
                        .database
                        .most_listened_tracks()
                        .iter()
                        .map(|song_id| self.database.get_song(song_id).title().to_string())
                        .collect::<Vec<String>>(),
                    _ => {
                        panic!("Should not reach")
                    }
                },
            },
            CurrentScreen::Albums => match self.album_pane {
                TwoPaneVertical::Left => self
                    .database
                    .filtered_albums()
                    .iter()
                    .map(|album_id| self.database.get_album(album_id).name().to_string())
                    .collect::<Vec<String>>(),
                TwoPaneVertical::Right => self
                    .database
                    .get_album(
                        self.database
                            .filtered_albums()
                            .get(self.list_states.album_state.selected().unwrap())
                            .unwrap(),
                    )
                    .songs()
                    .iter()
                    .map(|song_id| self.database.get_song(song_id).title().to_string())
                    .collect::<Vec<String>>(),
            },
            CurrentScreen::Playlists => match self.playlist_pane {
                TwoPaneVertical::Left => self
                    .database
                    .alphabetical_playlists()
                    .iter()
                    .map(|playlist_id| self.database.get_playlist(playlist_id).name().to_string())
                    .collect::<Vec<String>>(),
                TwoPaneVertical::Right => self
                    .database
                    .playlists()
                    .get(
                        self.database
                            .alphabetical_playlists()
                            .get(self.list_states.playlist_state.selected().unwrap())
                            .unwrap(),
                    )
                    .unwrap()
                    .song_list()
                    .iter()
                    .map(|song_id| self.database.get_song(song_id).title().to_string())
                    .collect::<Vec<String>>(),
            },
            CurrentScreen::Artists => match self.artist_pane {
                TwoPaneVertical::Left => self
                    .database
                    .alphabetical_artists()
                    .iter()
                    .map(|artist_id| self.database.get_artist(artist_id).name().to_string())
                    .collect::<Vec<String>>(),
                TwoPaneVertical::Right => {
                    let mut result: Vec<String> = vec![];
                    let selected_artist = self.database.get_artist(
                        self.database
                            .alphabetical_artists()
                            .get(self.list_states.artist_state.selected().unwrap())
                            .unwrap(),
                    );
                    for album_id in selected_artist.albums() {
                        result.push(self.database.get_album(album_id).name().to_string());
                        for song_id in self.database.get_album(album_id).songs() {
                            result.push(self.database.get_song(song_id).title().to_string());
                        }
                    }
                    result
                }
            },
            CurrentScreen::Queue => self
                .queue
                .iter()
                .map(|song_id| self.database.get_song(song_id).title().to_string())
                .collect::<Vec<String>>(),
        };

        self.app_flags.upper_case_search = false;
        for char in self.search_data.search_string.chars() {
            self.app_flags.upper_case_search = char.is_uppercase();
            if self.app_flags.upper_case_search {
                break;
            }
        }

        for (index, string) in list_to_be_searched.iter().enumerate() {
            let string_to_search = if self.app_flags.upper_case_search {
                string
            } else {
                &string.to_lowercase()
            };
            if string_to_search.contains(self.search_data.search_string.as_str()) {
                debug!(
                    "song title: {}, matched string: {}",
                    string_to_search, self.search_data.search_string
                );
                self.search_data.search_results_indexes.push(index);
            }
        }

        Ok(())
    }
    pub fn go_next_in_search(&mut self) -> AppResult<()> {
        let list_selected_state = match self.current_screen {
            CurrentScreen::Home => match self.home_pane {
                HomePane::Top => self.list_states.home_tab_top.selected().unwrap(),
                HomePane::TopLeft => self.list_states.home_tab_top_left.selected().unwrap(),
                HomePane::TopRight => self.list_states.home_tab_top_right.selected().unwrap(),
                HomePane::Bottom => self.list_states.home_tab_bottom.selected().unwrap(),
                HomePane::BottomLeft => self.list_states.home_tab_bottom_left.selected().unwrap(),
                HomePane::BottomRight => self.list_states.home_tab_bottom_right.selected().unwrap(),
            },
            CurrentScreen::Albums => match self.album_pane {
                TwoPaneVertical::Left => self.list_states.album_state.selected().unwrap(),
                TwoPaneVertical::Right => self.list_states.album_selected_state.selected().unwrap(),
            },
            CurrentScreen::Playlists => match self.playlist_pane {
                TwoPaneVertical::Left => self.list_states.playlist_state.selected().unwrap(),
                TwoPaneVertical::Right => {
                    self.list_states.playlist_selected_state.selected().unwrap()
                }
            },
            CurrentScreen::Artists => match self.artist_pane {
                TwoPaneVertical::Left => self.list_states.artist_state.selected().unwrap(),
                TwoPaneVertical::Right => {
                    self.list_states.artist_selected_state.selected().unwrap()
                }
            },
            CurrentScreen::Queue => self.list_states.queue_list_state.selected().unwrap(),
        };
        if self.search_data.search_results_indexes.is_empty() {
            return Ok(());
        }
        if self.search_data.index_in_search == usize::MAX {
            // If we index is equal to MAX, we are starting search
            // We will try to get the search result after the current cursor position
            self.search_data.index_in_search = 0;
            while self.search_data.search_results_indexes[self.search_data.index_in_search]
                < list_selected_state
            {
                if self.search_data.index_in_search
                    < self.search_data.search_results_indexes.len() - 1
                {
                    self.search_data.index_in_search += 1;
                } else {
                    break;
                }
            }
        } else if self.search_data.index_in_search
            == self.search_data.search_results_indexes.len() - 1
        {
            // If we are at the end, go back to beginning
            self.search_data.index_in_search = 0;
        } else {
            // Else go to next result
            self.search_data.index_in_search += 1;
        }
        self.app_flags.move_to_next_in_search = true;
        Ok(())
    }

    pub fn go_previous_in_search(&mut self) -> AppResult<()> {
        if self.search_data.search_results_indexes.is_empty() {
            return Ok(());
        }
        if self.search_data.index_in_search == usize::MAX || self.search_data.index_in_search == 0 {
            self.search_data.index_in_search = self.search_data.search_results_indexes.len() - 1;
        } else {
            self.search_data.index_in_search -= 1;
        }
        self.app_flags.move_to_next_in_search = true;
        Ok(())
    }

    pub fn clear_search(&mut self) -> AppResult<()> {
        self.search_data.search_string = "".to_string();
        self.search_data.search_results_indexes.clear();
        self.search_data.index_in_search = usize::MAX;
        self.app_flags.move_to_next_in_search = false;

        Ok(())
    }

    pub fn clear_search_results(&mut self) -> AppResult<()> {
        self.search_data.search_results_indexes.clear();
        self.search_data.index_in_search = usize::MAX;

        Ok(())
    }

    pub fn set_replay_gain(&mut self, replay_gain_mode: &str) -> AppResult<()> {
        self.player.set_replay_gain(replay_gain_mode);
        Ok(())
    }

    pub fn is_selected_playlist_local(&self) -> AppResult<bool> {
        Ok(self
            .database
            .alphabetical_playlists()
            .get(self.list_states.playlist_state.selected().unwrap())
            .unwrap()
            .starts_with("local"))
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

    pub fn delete_selected_playlist(&mut self) -> AppResult<()> {
        let playlist_id = self
            .database
            .alphabetical_playlists()
            .get(self.list_states.playlist_state.selected().unwrap())
            .unwrap()
            .clone();
        if !self.is_selected_playlist_local()? {
            self.server.delete_playlist_async(playlist_id.as_str());
        }
        self.database.remove_playlist(playlist_id.as_str());
        self.database
            .set_alphabetical_playlists(sort_playlists_by_name(self.database.playlists()));
        Ok(())
    }

    pub fn remove_albums_missing_in_server(&mut self) {
        let missing_albums = self
            .database
            .albums()
            .iter()
            .filter(|(album_id, _)| !self.database.alphabetical_list_albums().contains(album_id))
            .map(|(album_id, _)| album_id.clone())
            .collect::<Vec<_>>();
        for album_id in missing_albums {
            debug!("Album {} not found in server, deleting", album_id);
            self.database.remove_album(album_id.as_str());
        }
    }

    pub fn clear_errors_in_operations(&mut self) -> AppResult<()> {
        for operation in &mut self.server.operations {
            if operation.error() {
                operation.set_error(false);
                // We also clear the started flag to force retrying
                operation.set_started(false);
            }
        }
        Ok(())
    }

    pub fn process_pending_requests(&mut self) {
        self.server.process_async_operations();

        let pending_operations_number = self.server.operations.len();

        if pending_operations_number > 0 {
            self.status = AppStatus::Updating
        } else {
            self.status = AppStatus::Connected;
            if !(self.app_flags.updating_albums || self.app_flags.updating_alphabetical_albums)
                && self.app_flags.updating_database
            {
                self.finish_database_update();
                self.app_flags.updating_database = false;
            }
        }

        // We will prioritize fetching the alphabetical album list to ensure we have all albums
        // before anything else
        for i in 0..pending_operations_number {
            let operation = &mut self.server.operations[i];
            if operation.error() {
                debug!("Operation {:?} failed", operation.operation_id());
                self.status = AppStatus::Disconnected;
                self.current_popup = Popup::ConnectionError;
                break;
            }
            if operation.finished() && !operation.processed() {
                debug!(
                    "Processing finished operation {:?}, updating_albums: {}, updating_alphabetical_list: {}",
                    operation.operation_id(),
                    self.app_flags.updating_albums,
                    self.app_flags.updating_alphabetical_albums
                );
                match operation.operation_id() {
                    Operation::GetPlaylistList(update) => {
                        if self.app_flags.updating_albums
                            || self.app_flags.updating_alphabetical_albums
                        {
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
                        if self.app_flags.updating_albums
                            || self.app_flags.updating_alphabetical_albums
                        {
                            continue;
                        }
                        self.database.set_playlist_songs(
                            id,
                            Parser::parse_playlist(operation.result().to_string()).unwrap(),
                        );
                        // We have the latest version from server, so remove modified flag
                        self.database
                            .get_mut_playlist(id.as_str())
                            .set_modified(false);
                        operation.set_processed(true);
                    }
                    Operation::CreatePlaylist(temporary_id) => {
                        if self.app_flags.updating_albums
                            || self.app_flags.updating_alphabetical_albums
                        {
                            continue;
                        }
                        match Parser::parse_playlist_id(operation.result().to_string()) {
                            Ok(playlist_id) => {
                                let mut updated_playlist =
                                    self.database.remove_playlist(temporary_id);
                                updated_playlist.set_id(playlist_id.clone());
                                updated_playlist.set_modified(false);
                                self.database.insert_playlist(playlist_id, updated_playlist);
                                self.database
                                    .set_alphabetical_playlists(sort_playlists_by_name(
                                        self.database.playlists(),
                                    ));
                            }
                            Err(e) => {
                                warn!("Could not parse playlist id: {}", e);
                            }
                        }
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
                            }
                        }
                    }
                    Operation::GetAlbum(album_id) => {
                        let (album, songs, artist) =
                            Parser::parse_album(operation.result().to_string()).unwrap();
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
                        // album to it if it did not have it already.
                        if !self.database.contains_artist(artist.id()) {
                            debug!("Artist {} not in database, inserting", artist.name());
                            let artist_id = artist.id().to_string();
                            self.database.insert_artist(artist.id().to_string(), artist);
                            self.database
                                .get_artist_mut(artist_id.as_str())
                                .insert_album(album_id.clone(), album_genres);
                        } else if !self
                            .database
                            .get_artist(artist.id())
                            .albums()
                            .contains(album_id)
                        {
                            debug!(
                                "Artist {} already in database. Adding album {} to it",
                                artist.name(),
                                album_id
                            );
                            self.database
                                .get_artist_mut(artist.id())
                                .insert_album(album_id.clone(), album_genres);
                        } else {
                            debug!("Artist {} already had album {}", artist.name(), album_id);
                        }
                        self.albums_being_updated -= 1;
                        operation.set_processed(true);
                        // If there are no more albums being updated, and we are not updating the
                        // alphabetical list, we can be sure we have them all
                        if self.albums_being_updated == 0
                            && !self.app_flags.updating_alphabetical_albums
                        {
                            self.app_flags.updating_albums = false;
                        }
                    }
                    Operation::GetAlbumListRecent() => {
                        if self.app_flags.updating_albums
                            || self.app_flags.updating_alphabetical_albums
                        {
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
                        if self.app_flags.updating_albums
                            || self.app_flags.updating_alphabetical_albums
                        {
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
                        if self.app_flags.updating_albums
                            || self.app_flags.updating_alphabetical_albums
                        {
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
                    Operation::DeletePlaylist(playlist_id) => {
                        debug!(
                            "DeletePlaylist operation done for playlist: {}",
                            playlist_id
                        );
                        operation.set_processed(true);
                    }
                    Operation::UpdatePlaylist(playlist_id) => {
                        debug!(
                            "UpdatePlaylist operation done for playlist: {}",
                            playlist_id
                        );
                        self.database
                            .get_mut_playlist(playlist_id)
                            .set_modified(false);
                        operation.set_processed(true);
                    }
                }
            }
        }
        self.server.remove_completed_operations();
    }

    fn finish_database_update(&mut self) {
        if self.database.get_number_of_albums() > self.database.alphabetical_list_albums().len() {
            debug!("Number of albums in database ({}) is greater than alphabetical list ({}), albums have been deleted!",
                                    self.database.get_number_of_albums(), self.database.alphabetical_list_albums().len());
            self.remove_albums_missing_in_server()
        }
        self.process_filtered_album_list().unwrap();
        self.database
            .set_most_listened_tracks(sort_songs_by_play_count(self.database.songs()));
        self.database
            .set_alphabetical_artists(sort_artists_by_name(self.database.artists()));
        self.database
            .set_alphabetical_playlists(sort_playlists_by_name(self.database.playlists()));
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

fn sort_playlists_by_name(playlists: &HashMap<String, Playlist>) -> Vec<String> {
    let mut playlists_vec: Vec<(String, String)> = playlists
        .iter()
        .map(|(_, playlist)| (playlist.name().to_string(), playlist.id().to_string()))
        .collect();

    playlists_vec.sort_by(|a, b| a.0.cmp(&b.0));

    playlists_vec.into_iter().map(|(_, id)| id).collect()
}
