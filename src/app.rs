use crate::constants;
use crate::constants::{DEFAULT_ALBUM, DEFAULT_SONG};
use crate::event::DbusEvent::{Clear, Metadata, Paused, Playing, Stop};
use crate::event::Event;
use crate::event::Event::{Dbus, Draw};
use crate::mappings::Mappings;
use crate::model::artist::Artist;
use crate::model::playlist::Playlist;
use crate::model::song::Song;
use crate::music_database::MusicDatabase;
use crate::player::ipc::IpcEvent;
use crate::player::mpv::{Mpv, PlayerStatus};
use crate::player_data::{AppLoopStatus, PlayerData};
use crate::server::async_operation::Operation;
use crate::server::parser::Parser;
use crate::server::server::Server;
use chrono::NaiveDateTime;
use config::Config;
use log::{debug, error, info, warn};
use rand::seq::SliceRandom;
use rand::{rng, Rng};
use ratatui::prelude::Color;
use ratatui::widgets::ListState;
use secret_service::{SecretService, EncryptionType};
use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use std::error;
use std::process::exit;
use std::str::FromStr;
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

impl CurrentScreen {
    pub fn as_str(&self) -> &'static str {
        match self {
            CurrentScreen::Home => "Home",
            CurrentScreen::Albums => "Albums",
            CurrentScreen::Playlists => "Playlists",
            CurrentScreen::Artists => "Artists",
            CurrentScreen::Queue => "Queue",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AppStatus {
    Connected,
    Disconnected,
    Updating,
}

#[derive(Debug, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortOrder::Ascending => "ascending",
            SortOrder::Descending => "descending",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SortMode {
    Frequent,
    Alphabetical,
}

impl SortMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortMode::Frequent => "frequent",
            SortMode::Alphabetical => "alphabetical",
        }
    }
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
    pub fn as_str(&self) -> &'static str {
        match self {
            HomePane::Top => "top",
            HomePane::TopLeft => "top_left",
            HomePane::TopRight => "top_right",
            HomePane::Bottom => "bottom",
            HomePane::BottomLeft => "bottom_left",
            HomePane::BottomRight => "bottom_right",
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
    pub fn as_str(&self) -> &'static str {
        match self {
            TwoPaneVertical::Left => "left",
            TwoPaneVertical::Right => "right",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FourPaneGrid {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
// Implement a method to cast the enum to a u8
impl FourPaneGrid {
    pub fn to_u8(&self) -> u8 {
        match self {
            FourPaneGrid::TopLeft => 0,
            FourPaneGrid::TopRight => 1,
            FourPaneGrid::BottomLeft => 2,
            FourPaneGrid::BottomRight => 3,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            FourPaneGrid::TopLeft => "top_left",
            FourPaneGrid::TopRight => "top_right",
            FourPaneGrid::BottomLeft => "bottom_left",
            FourPaneGrid::BottomRight => "bottom_right",
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
    YearFilter,
    UpdateDatabase,
    SelectPlaylist,
    SynchronizePlaylist,
    ConfirmPlaylistDeletion,
    None,
    ConnectionError,
    GlobalSearch,
    ErrorMessage,
}

impl Popup {
    pub fn as_str(&self) -> &'static str {
        match self {
            Popup::ConnectionTest => "connection_test",
            Popup::AlbumInformation => "album_information",
            Popup::AddTo => "add_to",
            Popup::GenreFilter => "genre_filter",
            Popup::YearFilter => "year_filter",
            Popup::UpdateDatabase => "update_database",
            Popup::SelectPlaylist => "select_playlist",
            Popup::SynchronizePlaylist => "synchronize_playlist",
            Popup::ConfirmPlaylistDeletion => "confirm_playlist_deletion",
            Popup::ConnectionError => "connection_error",
            Popup::None => "none",
            Popup::GlobalSearch => "global_search",
            Popup::ErrorMessage => "error_message",
        }
    }
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
    pub player: Mpv,
    pub ticks_during_playing_state: usize,
    pub album_filters: AlbumFilters,
    pub album_sorting_mode: SortMode,
    pub album_sorting_direction: SortOrder,
    pub search_data: SearchData,
    pub status: AppStatus,
    pub result_list_alphabetical: Vec<String>,
    pub result_list_most_listened: Vec<String>,
    pub albums_being_updated: usize,
    pub album_pane: TwoPaneVertical,
    pub artist_pane: TwoPaneVertical,
    pub playlist_pane: TwoPaneVertical,
    pub new_name: String,
    pub selected_album_id_to_update: String,
    pub player_data: PlayerData,
    pub app_colors: AppColors,
    pub app_config: AppConfig,
    pub app_focused: bool,
    pub shortcuts: Mappings,
    pub global_search_pane: FourPaneGrid,
    pub error_message: String,
}

#[derive(Default, Debug)]
pub struct AppConfig {
    pub list_size: usize,
    pub follow_cursor: bool,
    pub draw_while_unfocused: bool,
    pub save_player_status: bool,
    pub wait_for_ipc_ms: u64,
    pub album_list_api_namespace: String,
    pub reorder_random_queue: bool,
    pub parser_type: Parser
}

pub struct AlbumFilters {
    pub genre_filter: String,
    pub year_from_filter: String,
    pub year_to_filter: String,
    pub year_from_filter_new: String,
    pub year_to_filter_new: String,
    pub filter_message: String,
}

impl AlbumFilters {
    fn default() -> Self {
        AlbumFilters {
            genre_filter: String::from("any"),
            year_from_filter: String::from(""),
            year_to_filter: String::from(""),
            year_from_filter_new: String::from(""),
            year_to_filter_new: String::from(""),
            filter_message: String::from(""),
        }
    }
}

#[derive(Default, Debug)]
pub struct ItemToBeAdded {
    pub name: String,
    pub id: String,
    pub parent_id: String,
    pub media_type: MediaType,
}

pub struct AppColors {
    pub primary_accent: Color,
    pub secondary_accent: Color,
    pub connected: Color,
    pub updating: Color,
    pub error: Color,
    pub now_playing: Color,
}

impl AppColors {
    fn default() -> Self {
        AppColors {
            primary_accent: Color::Yellow,
            secondary_accent: Color::Gray,
            connected: Color::Green,
            updating: Color::Yellow,
            error: Color::Red,
            now_playing: Color::Green,
        }
    }
}

pub struct SearchData {
    pub global_search_string: String,
    pub search_string: String,
    pub index_in_search: usize,
    pub search_results_indexes: Vec<usize>,
    pub global_search_song_results: Vec<String>,
    pub global_search_albums_results: Vec<String>,
    pub global_search_playlists_results: Vec<String>,
    pub global_search_artists_results: Vec<String>,
}

impl Default for SearchData {
    fn default() -> Self {
        SearchData {
            global_search_string: String::from(""),
            search_string: String::from(""),
            index_in_search: usize::MAX,
            search_results_indexes: Vec::new(),
            global_search_song_results: Vec::new(),
            global_search_albums_results: Vec::new(),
            global_search_playlists_results: Vec::new(),
            global_search_artists_results: Vec::new(),
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct AppFlags {
    pub running: bool,
    pub getting_search_string: bool,
    pub move_to_next_in_search: bool,
    pub upper_case_search: bool,
    pub updating_database: bool,
    pub updating_albums: bool,
    pub updating_alphabetical_albums: bool,
    pub replay_gain_auto: bool,
    pub is_current_song_scrobbled: bool,
    pub is_introducing_new_playlist_name: bool,
    pub is_introducing_to_year: bool,
    pub is_introducing_global_search: bool,
    pub range_year_filter: bool,
    pub seeking: bool,
    pub was_paused: bool,
    pub invalidate_search: bool,
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
    pub global_search_songs: ListState,
    pub global_search_albums: ListState,
    pub global_search_artists: ListState,
    pub global_search_playlists: ListState,
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
            global_search_songs: AppListStates::initialize(),
            global_search_albums: AppListStates::initialize(),
            global_search_artists: AppListStates::initialize(),
            global_search_playlists: AppListStates::initialize(),
        }
    }

    fn initialize() -> ListState {
        let mut list_state = ListState::default();
        list_state.select_first();
        list_state
    }
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
            player: Mpv::default(),
            player_data: PlayerData::default(),
            ticks_during_playing_state: 0,
            album_filters: AlbumFilters::default(),
            album_sorting_mode: SortMode::Alphabetical,
            album_sorting_direction: SortOrder::Descending,
            search_data: SearchData::default(),
            status: AppStatus::Connected,
            result_list_most_listened: vec![],
            result_list_alphabetical: vec![],
            albums_being_updated: 0,
            album_pane: TwoPaneVertical::Left,
            artist_pane: TwoPaneVertical::Left,
            playlist_pane: TwoPaneVertical::Left,
            new_name: String::from(""),
            selected_album_id_to_update: String::from(""),
            app_colors: AppColors::default(),
            app_config: AppConfig::default(),
            app_focused: true,
            shortcuts: Mappings::default(),
            global_search_pane: FourPaneGrid::TopLeft,
            error_message: "".to_string(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) -> AppResult<()> {
        if self.app_flags.invalidate_search {
            self.clear_search_results()?;
            self.app_flags.invalidate_search = false;
        }
        if self.status != AppStatus::Disconnected && self.mode != AppConnectionMode::Offline {
            self.process_pending_requests();
        }
        self.process_player_events();
        if *self.player.player_status() == PlayerStatus::Playing {
            self.ticks_during_playing_state += 1;
            // If we have only 10 seconds left for the current track
            if self.get_playback_time() + 10
                > self
                    .player_data
                    .now_playing
                    .duration
                    .as_str()
                    .parse::<usize>()
                    .unwrap()
            {
                // If there is a next element in queue, add it to mpv queue if it has not been yet added
                if !self.player_data.next_is_in_player_queue && self.queue_has_next() {
                    let next_index = self
                        .player_data
                        .queue_order
                        .get(self.player_data.index_in_queue + 1)
                        .unwrap();
                    self.player.add_next_song_to_queue(
                        self.server
                            .get_song_url(self.player_data.queue.get(*next_index).unwrap().clone())
                            .as_str(),
                    );
                    self.player_data.next_is_in_player_queue = true;
                }

                if !self.app_flags.is_current_song_scrobbled {
                    // Update last listened album id to remember it for next startup
                    let now_playing_album_id = self
                        .database
                        .get_song(self.player_data.now_playing.id.as_str())
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
                    self.server
                        .scrobble_song_async(self.player_data.now_playing.id.clone());
                    self.increase_play_count_for_current_song_in_database(
                        self.player_data.now_playing.id.clone(),
                    );
                    self.app_flags.is_current_song_scrobbled = true;
                }
            }
        }

        Ok(())
    }

    /// Set running to false in order to quit the application.
    pub fn quit(&mut self) {
        self.player.quit_player();
        self.player_data.player_volume = self.player.get_volume();
        self.app_flags.running = false;
    }

    pub fn restore_volume(&mut self) {
        self.player.set_volume(self.player_data.player_volume);
    }

    pub async fn set_config(&mut self, config: Config) -> AppResult<()> {
        match config.get::<String>("server_address") {
            Ok(address) => self.server.server_address = address,
            Err(e) => {
                println!("Could not load server address.");
                error!("Failed to load server address. {}", e);
                exit(1)
            }
        }
        match config.get::<String>("user") {
            Ok(user) => self.server.user = user,
            Err(e) => {
                println!("Could not load username.");
                error!("Failed to load user name. {}", e);
                exit(1)
            }
        }
        match config.get::<String>("server_auth") {
            Ok(auth_mode) => {
                if auth_mode == "token" || auth_mode == "plain" {
                    self.server.server_auth = auth_mode
                }
            }
            Err(e) => {
                warn!("Could not parse auth mode {}", e);
                info!("Using default server auth mode: token.");
                self.server.server_auth = "token".to_string();
            }
        }
        match config.get::<String>("password_store") {
            Ok(store) => self.set_password_from_store(&config, store).await?,
            Err(e) => {
                warn!("Could not parse password store {}", e);
                info!("Using default password store: plain.");
                self.set_password_from_store(&config, "plain".to_string()).await?
            }
        }

        match config.get::<String>("album_list_api") {
            Ok(auth_mode) => {
                if auth_mode == "v2" {
                    info!("Using album list API version: v2.");
                    self.server.album_lists_api = "getAlbumList2".to_string();
                    self.app_config.album_list_api_namespace = "albumList2".to_string();
                } else {
                    info!("Using album list API version: v1.");
                    self.app_config.album_list_api_namespace = "albumList".to_string();
                    self.server.album_lists_api = "getAlbumList".to_string();
                }
            }
            Err(e) => {
                warn!("Could not parse album list version {}", e);
                info!("Using default album list API version: v1.");
                self.server.album_lists_api = "getAlbumList".to_string();
                self.app_config.album_list_api_namespace = "albumList".to_string();
            }
        }
        match config.get::<u64>("wait_for_ipc_ms") {
            Ok(wait_time) => {
                self.app_config.wait_for_ipc_ms = wait_time;
            }
            Err(e) => {
                warn!("Could not parse wait time for ipc, using default. {}", e);
                self.app_config.wait_for_ipc_ms = 200;
            }
        }
        if let Ok(color) = config.get::<String>("primary_accent") {
            match parse_color(color.as_str()) {
                Ok(parsed_color) => self.app_colors.primary_accent = parsed_color,
                Err(_) => warn!("Could not parse primary color from {}", color),
            }
        }
        if let Ok(color) = config.get::<String>("secondary_accent") {
            match parse_color(color.as_str()) {
                Ok(parsed_color) => self.app_colors.secondary_accent = parsed_color,
                Err(_) => warn!("Could not parse secondary color from {}", color),
            }
        }

        match config.get::<usize>("home_list_size") {
            Ok(value) => self.app_config.list_size = value,
            Err(e) => {
                self.app_config.list_size = 20;
                warn!("Could not load home size size, using default. {}", e);
            }
        }

        match config.get::<bool>("follow_cursor_queue") {
            Ok(value) => self.app_config.follow_cursor = value,
            Err(e) => {
                warn!("Could not load follow cursor queue, using default. {}", e);
                self.app_config.follow_cursor = true;
            }
        }

        match config.get::<bool>("draw_while_unfocused") {
            Ok(value) => self.app_config.draw_while_unfocused = value,
            Err(e) => {
                warn!(
                    "Could not load draw while unfocused, while draw always. {}",
                    e
                );
                self.app_config.draw_while_unfocused = true;
            }
        }

        match config.get::<bool>("save_player_status") {
            Ok(value) => self.app_config.save_player_status = value,
            Err(e) => {
                info!(
                    "Could not load option to save player status, will not save by default. {}",
                    e
                );
                self.app_config.save_player_status = false;
            }
        }

        match config.get::<bool>("reorder_random_queue") {
            Ok(value) => self.app_config.reorder_random_queue = value,
            Err(e) => {
                info!(
                    "Could not load option to to reorder random queue, will not reorder queue. {}",
                    e
                );
                self.app_config.reorder_random_queue = false;
            }
        }

        match config.get::<String>("parser_type") {
            Ok(parser_type) => {
                if parser_type == "json" {
                    info!("Using parser type: json.");
                    self.app_config.parser_type = Parser::JsonParser;
                } else if parser_type == "xml" {
                    info!("Using parser type: xml.");
                    self.app_config.parser_type = Parser::XmlParser;
                    self.server.json_parser = false;
                }
                else {
                    warn!("Unknown parser type {}", parser_type);
                    info!("Using default parser type: json.");
                    self.app_config.parser_type = Parser::JsonParser;
                }
            }
            Err(e) => {
                warn!("Could not parse parser type {}", e);
                info!("Using default parser type: json.");
                self.app_config.parser_type = Parser::JsonParser;
            }
        }

        self.shortcuts.init_shortcuts(config);

        Ok(())
    }

    async fn set_password_from_store(&mut self, config: &Config, store: String) -> AppResult<()> {
        match store.as_str() {
            "secretservice" => {
                self.set_secret_service_password().await?
            }
            "plain" | _ => match config.get::<String>("password") {
                Ok(password) => self.server.set_password(password),
                Err(e) => {
                    println!("Could not load password.");
                    error!("Failed to load password. {}", e);
                    exit(1)
                }
            },
        }
        Ok(())
    }

    async fn set_secret_service_password(&mut self) -> AppResult<()> {
        let mut authorized = false;
        while !authorized {
            let store = SecretService::connect(EncryptionType::Dh).await?;
            let collection = store.get_default_collection().await?;
            if collection.is_locked().await? {
                collection.unlock().await?;
            };
            let attributes = HashMap::from([
                ("app_id", "com.gitlab.detoxify92.naviterm"),
                ("server", &self.server.server_address),
                ("username", &self.server.user),
            ]);
            let search_items = collection.search_items(attributes.clone()).await?;
            let item = match search_items.first() {
                Some(i) => i,
                None => {
                    let prompt = format!(
                        "Please enter password for {}@{}: ",
                        self.server.user,
                        self.server.server_address
                    );
                    let secret = rpassword::prompt_password(prompt)?;
                    &collection.create_item(
                        "Naviterm",
                        attributes,
                        secret.trim().as_bytes(),
                        false,
                        "text/plain"
                    ).await?
                },
            };
            let secret = item.get_secret().await?;
            self.server.set_password(String::from_utf8(secret)?);

            authorized = true;

            // Check if credentials are valid
            self.renew_credentials()?;
            self.test_connection().await?;

            // 40 represents "Wrong username or password." in the subsonic API.
            if self.server.connection_code == "40" {
                authorized = false;

                error!("Invalid credentials, retry");
                println!("Invalid credentials, retry");

                item.delete().await?
            }
        }
        Ok(())
    }

    pub fn renew_credentials(&mut self) -> AppResult<()> {
        self.server.renew_credentials()?;
        Ok(())
    }

    pub async fn test_connection(&mut self) -> AppResult<()> {
        self.server.test_connection(self.app_config.parser_type).await?;
        Ok(())
    }

    pub fn check_server_connection_status(&self) -> bool {
        if self.server.connection_status == "failed" {
            println!(
                "Server connection failed. Error code: {}, error message: {}",
                self.server.connection_code, self.server.connection_message
            );
            error!(
                "Server connection failed. Error code: {}, error message: {}",
                self.server.connection_code, self.server.connection_message
            );
            false
        } else {
            true
        }
    }
    pub fn populate_db(&mut self, force_update: bool) -> AppResult<()> {
        info!(
            "Starting database population. Force update: {}",
            force_update
        );
        // Used to prioritize album updating vs other operations that would mean looking for an
        // album we have not yet fetched
        self.app_flags.updating_albums = true;
        self.app_flags.updating_database = true;
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

    pub fn initialize_player(&mut self, mpv_custom_args: Vec<String>) -> AppResult<()> {
        self.player.create_player(mpv_custom_args)?;
        Ok(())
    }

    pub fn initialize_player_stream(&mut self) -> AppResult<()> {
        match self.player.initialize() {
            Ok(_) => Ok(()),
            Err(_) => {
                warn!("Could not initialize ipc stream, retrying...");
                sleep(Duration::from_millis(self.app_config.wait_for_ipc_ms));
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
        if self.item_to_be_added.id.is_empty() {
            warn!("Item to be added empty, could not add queue immediately.");
            return Ok(());
        }
        self.player_data.index_in_queue = 0;
        match self.item_to_be_added.media_type {
            MediaType::Song => {
                self.player_data.queue.clear();
                self.player_data.queue_order.clear();
                self.player_data
                    .queue
                    .push(self.item_to_be_added.id.clone());
                self.player_data
                    .queue_order
                    .push(self.player_data.queue.len() - 1);
                self.change_current_playing_to(self.item_to_be_added.id.clone().as_str());
            }
            MediaType::Album => {
                self.player_data.queue.clear();
                self.player_data.queue_order.clear();
                let album = self.database.get_album(self.item_to_be_added.id.as_str());
                for song in album.songs() {
                    self.player_data.queue.push(song.clone());
                }
                self.player_data.queue_order = (0..self.player_data.queue.len()).collect();
                if self.player_data.random_playback {
                    let mut rng = rng();
                    let random_index = rng.random_range(0..self.player_data.queue.len());
                    self.shuffle_queue_order_starting_at_index(random_index);
                }
                self.change_current_playing_to(
                    self.player_data.queue[self.player_data.queue_order[0]]
                        .clone()
                        .as_str(),
                );
            }
            MediaType::Playlist => {
                self.player_data.queue.clear();
                self.player_data.queue_order.clear();
                for song_id in self
                    .database
                    .get_playlist(self.item_to_be_added.id.as_str())
                    .song_list()
                {
                    self.player_data.queue.push(song_id.clone());
                }
                self.player_data.queue_order = (0..self.player_data.queue.len()).collect();
                if self.player_data.random_playback {
                    let mut rng = rng();
                    let random_index = rng.random_range(0..self.player_data.queue.len());
                    self.shuffle_queue_order_starting_at_index(random_index);
                }
                self.change_current_playing_to(
                    self.player_data.queue[self.player_data.queue_order[0]]
                        .clone()
                        .as_str(),
                );
            }
            MediaType::Artist => {
                self.player_data.queue.clear();
                self.player_data.queue_order.clear();
                for album_id in self
                    .database
                    .get_artist(self.item_to_be_added.id.as_str())
                    .albums()
                {
                    let album = self.database.get_album(album_id.as_str());
                    for song in album.songs() {
                        self.player_data.queue.push(song.clone());
                    }
                }
                self.player_data.queue_order = (0..self.player_data.queue.len()).collect();
                if self.player_data.random_playback {
                    let mut rng = rng();
                    let random_index = rng.random_range(0..self.player_data.queue.len());
                    self.shuffle_queue_order_starting_at_index(random_index);
                }
                self.change_current_playing_to(
                    self.player_data.queue[self.player_data.queue_order[0]]
                        .clone()
                        .as_str(),
                );
            }
        }
        self.update_queue_data();
        self.player.restore_player();
        self.play_current(false);
        Ok(())
    }

    pub fn add_queue_next(&mut self) -> AppResult<()> {
        let mut was_empty = false;
        let mut index_to_insert_to = if self.player_data.queue.is_empty() {
            debug!("Queue was empty");
            was_empty = true;
            0
        } else {
            self.player_data
                .queue
                .iter()
                .position(|x| x == &self.player_data.now_playing.id)
                .unwrap()
                + 1
        };
        let mut index_in_queue_order = if self.player_data.queue.is_empty() {
            0
        } else {
            self.player_data.index_in_queue + 1
        };
        match self.item_to_be_added.media_type {
            MediaType::Song => {
                self.player_data
                    .queue
                    .insert(index_to_insert_to, self.item_to_be_added.id.clone());
                update_queue_order_when_adding_next(&mut self.player_data, index_in_queue_order, index_to_insert_to);
            }
            MediaType::Album => {
                let album = self.database.get_album(self.item_to_be_added.id.as_str());
                for song in album.songs() {
                    self.player_data
                        .queue
                        .insert(index_to_insert_to, song.clone());
                    update_queue_order_when_adding_next(&mut self.player_data, index_in_queue_order, index_to_insert_to);
                    index_to_insert_to += 1;
                    index_in_queue_order += 1;
                }
            }
            MediaType::Playlist => {
                for song_id in self
                    .database
                    .get_playlist(self.item_to_be_added.id.as_str())
                    .song_list()
                {
                    self.player_data
                        .queue
                        .insert(index_to_insert_to, song_id.clone());
                    update_queue_order_when_adding_next(&mut self.player_data, index_in_queue_order, index_to_insert_to);
                    index_to_insert_to += 1;
                    index_in_queue_order += 1;
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
                        self.player_data
                            .queue
                            .insert(index_to_insert_to, song.clone());
                        update_queue_order_when_adding_next(&mut self.player_data, index_in_queue_order, index_to_insert_to);
                        index_in_queue_order += 1;
                        index_to_insert_to += 1;
                    }
                }
            }
        }
        if was_empty && !self.player_data.queue.is_empty() {
            self.change_current_playing_to(
                self.player_data.queue.first().unwrap().clone().as_str(),
            );
        }
        self.update_queue_data();
        Ok(())
    }

    pub fn add_queue_later(&mut self) -> AppResult<()> {
        let was_empty = self.player_data.queue.is_empty();
        match self.item_to_be_added.media_type {
            MediaType::Song => {
                self.player_data
                    .queue
                    .push(self.item_to_be_added.id.clone());
                self.player_data
                    .queue_order
                    .push(self.player_data.queue.len() - 1);
            }
            MediaType::Album => {
                let album = self.database.get_album(self.item_to_be_added.id.as_str());
                for song in album.songs() {
                    self.player_data.queue.push(song.clone());
                    self.player_data
                        .queue_order
                        .push(self.player_data.queue.len() - 1);
                }
            }
            MediaType::Playlist => {
                for song_id in self
                    .database
                    .get_playlist(self.item_to_be_added.id.as_str())
                    .song_list()
                {
                    self.player_data.queue.push(song_id.clone());
                    self.player_data
                        .queue_order
                        .push(self.player_data.queue.len() - 1);
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
                        self.player_data.queue.push(song.clone());
                        self.player_data
                            .queue_order
                            .push(self.player_data.queue.len() - 1);
                    }
                }
            }
        }
        if was_empty && !self.player_data.queue.is_empty() {
            self.change_current_playing_to(
                self.player_data.queue.first().unwrap().clone().as_str(),
            );
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
        if songs_to_add.is_empty() {
            return Ok(())
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
            new_playlist.set_created_on(chrono::Local::now().format("%m/%d/%y - %H:%M").to_string());
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
            playlist.song_list_mut().append(&mut songs_to_add);
            playlist.set_duration((duration + duration_to_add).to_string());
            playlist.set_song_count(playlist.song_list().len().to_string());
            playlist.set_modified_on(chrono::Local::now().format("%m/%d/%y - %H:%M").to_string());
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
        for (i, index_in_order_queue) in self.player_data.queue_order.iter().enumerate() {
            let song = self
                .database
                .get_song(self.player_data.queue.get(*index_in_order_queue).unwrap());
            duration_total += song.duration().parse::<usize>().unwrap();
            if i >= self.player_data.index_in_queue {
                duration_left += song.duration().parse::<usize>().unwrap();
            }
        }

        self.player_data.duration_total = duration_total.to_string();
        self.player_data.duration_left = duration_left.to_string();
    }
    
    pub fn global_search_set_item_to_be_added(&mut self) -> AppResult<()> {
        match self.global_search_pane {
            FourPaneGrid::TopLeft => {
                if self.search_data.global_search_song_results.is_empty() {
                    return Err("Search results for songs is empty".into());
                }
                let song = self.database.get_song(
                    self.search_data
                        .global_search_song_results
                        .get(self.list_states.global_search_songs.selected().unwrap())
                        .unwrap(),
                );
                self.item_to_be_added.id = song.id().to_string();
                self.item_to_be_added.media_type = MediaType::Song;
                self.item_to_be_added.name = song.title().to_string();
                self.item_to_be_added.parent_id = song.album_id().to_string();
            }
            FourPaneGrid::TopRight => {
                if self.search_data.global_search_albums_results.is_empty() {
                    return Err("Album results for songs is empty".into());
                }
                let album = self.database.get_album(
                    self.search_data
                        .global_search_albums_results
                        .get(self.list_states.global_search_albums.selected().unwrap())
                        .unwrap(),
                );
                self.item_to_be_added.id = album.id().to_string();
                self.item_to_be_added.media_type = MediaType::Album;
                self.item_to_be_added.name = album.name().to_string();
            }
            FourPaneGrid::BottomLeft => {
                if self.search_data.global_search_playlists_results.is_empty() {
                    return Err("Playlist results for songs is empty".into());
                }
                let playlist = self.database.get_playlist(
                    self.search_data
                        .global_search_playlists_results
                        .get(self.list_states.global_search_playlists.selected().unwrap())
                        .unwrap(),
                );
                self.item_to_be_added.name = playlist.name().to_string();
                self.item_to_be_added.id = playlist.id().to_string();
                self.item_to_be_added.media_type = MediaType::Playlist;
            }
            FourPaneGrid::BottomRight => {
                if self.search_data.global_search_artists_results.is_empty() {
                    return Err("Artists results for songs is empty".into());
                }
                let artist = self.database.get_artist(
                    self.search_data
                        .global_search_artists_results
                        .get(self.list_states.global_search_artists.selected().unwrap())
                        .unwrap(),
                );
                self.item_to_be_added.name = artist.name().to_string();
                self.item_to_be_added.id = artist.id().to_string();
                self.item_to_be_added.media_type = MediaType::Artist;
            }
        }

        Ok(())
    }

    pub fn go_to_according_pane_for_search_item(&mut self) -> AppResult<()> {
        match self.global_search_pane {
            FourPaneGrid::TopLeft => {
                if self.search_data.global_search_song_results.is_empty() {
                    return Err("Search results for songs is empty".into());
                }
                let song = self.database.get_song(
                    self.search_data
                        .global_search_song_results
                        .get(self.list_states.global_search_songs.selected().unwrap())
                        .unwrap(),
                );

                let album = self.database.get_album(song.album_id());

                let index = album
                    .songs()
                    .iter()
                    .position(|song_id| song_id == song.id());
                match index {
                    Some(index) => {
                        self.list_states.album_selected_state.select(Some(index));
                    }
                    None => {
                        warn!("Could not find the index for the song in the global search!")
                    }
                }
                self.set_album_index_for_album_id(album.id().to_string().as_str())?;
                self.album_pane = TwoPaneVertical::Right;
                self.current_screen = CurrentScreen::Albums;
            }
            FourPaneGrid::TopRight => {
                if self.search_data.global_search_albums_results.is_empty() {
                    return Err("Album results for songs is empty".into());
                }
                let album_id = self
                    .database
                    .get_album(
                        self.search_data
                            .global_search_albums_results
                            .get(self.list_states.global_search_albums.selected().unwrap())
                            .unwrap(),
                    )
                    .id()
                    .to_string();
                self.set_album_index_for_album_id(album_id.as_str())?;
                self.album_pane = TwoPaneVertical::Left;
                self.current_screen = CurrentScreen::Albums;
            }
            FourPaneGrid::BottomLeft => {
                if self.search_data.global_search_playlists_results.is_empty() {
                    return Err("Playlist results for songs is empty".into());
                }
                let playlist = self.database.get_playlist(
                    self.search_data
                        .global_search_playlists_results
                        .get(self.list_states.global_search_playlists.selected().unwrap())
                        .unwrap(),
                );
                let index = self
                    .database
                    .alphabetical_playlists()
                    .iter()
                    .position(|playlist_id| playlist_id == playlist.id());
                match index {
                    Some(index) => {
                        self.list_states.playlist_state.select(Some(index));
                    }
                    None => {
                        warn!("Could not find the index for the playlist in the global search!")
                    }
                }
                self.playlist_pane = TwoPaneVertical::Left;
                self.current_screen = CurrentScreen::Playlists;
            }
            FourPaneGrid::BottomRight => {
                if self.search_data.global_search_artists_results.is_empty() {
                    return Err("Artists results for songs is empty".into());
                }
                let artist = self.database.get_artist(
                    self.search_data
                        .global_search_artists_results
                        .get(self.list_states.global_search_artists.selected().unwrap())
                        .unwrap(),
                );
                let index = self
                    .database
                    .alphabetical_artists()
                    .iter()
                    .position(|artist_id| artist_id == artist.id());
                match index {
                    Some(index) => {
                        self.list_states.artist_state.select(Some(index));
                    }
                    None => {
                        warn!("Could not find the index for the artist in the global search!")
                    }
                }
                self.playlist_pane = TwoPaneVertical::Left;
                self.current_screen = CurrentScreen::Artists;
            }
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
                        if album_id_selected == DEFAULT_ALBUM {
                            warn!("Cannot add default album!");
                            return Err("Cannot add default album".into());
                        }
                        selected_album_index = 0;
                        for (i, album_id) in self.database.alphabetical_list_albums().iter().enumerate()
                        {
                            if album_id_selected == album_id {
                                selected_album_index = i;
                            }
                        }
                        self.database.alphabetical_list_albums()
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
                let albums = self
                    .database
                    .get_artist(
                        self.database
                            .alphabetical_artists()
                            .get(self.list_states.artist_state.selected().unwrap())
                            .unwrap(),
                    )
                    .albums();
                (selected_album_index, offset) =
                    self.get_selected_album_index_in_artist_view(albums);
                albums
            }
            _ => {
                selected_album_index = 0;
                self.database.filtered_albums()
            }
        };

        match media {
            MediaType::Song => {
                let selected_album_id = album_list.get(selected_album_index).ok_or("Could not find album for selected item")?;
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
                if song.id() == DEFAULT_SONG {
                    warn!("Cannot add default song!");
                    return Err("Cannot add default song!".into());
                }
                self.item_to_be_added.name = song.title().to_string();
                self.item_to_be_added.id = song.id().to_string();
                self.item_to_be_added.parent_id = selected_album_id.to_string();
                self.item_to_be_added.media_type = MediaType::Song;
            }
            MediaType::Album => {
                let album = self
                    .database
                    .get_album(album_list.get(selected_album_index).ok_or("Could not find album for selected item")?);
                if album.id() == DEFAULT_ALBUM {
                    warn!("Cannot add default album!");
                    return Err("Cannot add default album!".into());
                }
                self.item_to_be_added.id = album.id().to_string();
                self.item_to_be_added.name = album.name().to_string();
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
                                .ok_or("Could not find playlist for selected item")?,
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

    fn get_selected_album_index_in_artist_view(&self, albums: &Vec<String>) -> (usize, usize) {
        let mut offset = 0;
        let mut selected_album_index = 0;
        // Very hacky way of getting the album index, since the list of the album and songs
        // for the selected artist is not stored anywhere
        for (i, album_id) in albums.iter().enumerate() {
            let album = self.database.get_album(album_id.as_str());
            // The list also have the album title as elements, that is why we add 1 more
            offset += album.songs().len() + 1;
            if self.list_states.artist_selected_state.selected().unwrap() < offset {
                selected_album_index = i;
                // We will need this later
                offset -= album.songs().len();
                break;
            }
        }

        (selected_album_index, offset)
    }

    pub fn toggle_playing_status(&mut self) -> AppResult<()> {
        self.player.toggle_play_pause();
        if !self.app_focused {
            self.event_sender
                .as_ref()
                .unwrap()
                .send(Draw(true))
                .unwrap();
        }
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
        if self.is_selected_playlist_local()? {
            self.server
                .create_playlist_async(playlist.name(), playlist.song_list().clone(), playlist.id());
        } else {
            self.server
                .update_playlist_async(playlist.song_list().clone(), playlist.id());
        }
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
        if self.player_data.queue.len() > 1 {
            if self.player_data.random_playback {
                self.player_data.index_in_queue = *self
                    .player_data
                    .queue_order
                    .get(self.player_data.index_in_queue)
                    .unwrap();
                self.player_data.queue_order.clear();
                self.player_data.queue_order = (0..self.player_data.queue.len()).collect();
            } else {
                self.shuffle_queue_order_starting_at_index(self.player_data.index_in_queue);
                self.player_data.index_in_queue = 0;
            }
        }
        if self.app_flags.replay_gain_auto {
            if self.player_data.random_playback {
                self.player.set_replay_gain("album");
            } else {
                self.player.set_replay_gain("track");
            }
        }
        self.player_data.random_playback = !self.player_data.random_playback;
        self.update_queue_data();
        self.player_data.next_is_in_player_queue = false;
        Ok(())
    }

    pub fn player_seek_forward(&mut self) -> AppResult<()> {
        if self.player.player_status == PlayerStatus::Stopped {
            return Ok(());
        }
        if self.get_playback_time() + 10
            > self
                .player_data
                .now_playing
                .duration
                .as_str()
                .parse::<usize>()
                .unwrap()
        {
            self.play_next()?;
        } else {
            self.player.seek_forward();
            self.ticks_during_playing_state += 40;
        }
        Ok(())
    }

    pub fn player_seek_backwards(&mut self) -> AppResult<()> {
        if self.player.player_status == PlayerStatus::Stopped {
            return Ok(());
        }
        self.player.seek_backwards();
        self.ticks_during_playing_state = self.ticks_during_playing_state.saturating_sub(40);
        Ok(())
    }

    pub fn play_next(&mut self) -> AppResult<()> {
        if self.queue_has_next() {
            self.go_next_queue();
            self.play_current(false);
        } else if self.player_data.loop_status == AppLoopStatus::Playlist {
            debug!("Looping to first element in playlist");
            self.go_first_in_queue();
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
            let event = events.remove(0);
            debug!("Processing event: {:?}", event);
            match event {
                IpcEvent::FileLoaded => {
                    debug!("File loaded, buffering file");
                    self.set_player_to_buffering();
                }
                IpcEvent::PropertyChange(name, data) => {
                    if name == "pause" {
                        if data == "yes" && self.player.player_status == PlayerStatus::Playing {
                            debug!("Detected pause status change while playing, setting status to buffering...");
                            self.set_player_to_buffering();
                        } else if data == "no"
                            && self.player.player_status == PlayerStatus::Buffering
                        {
                            debug!("Detected pause status change while buffering, setting status to playing...");
                            self.set_player_to_playing();
                        } else {
                            debug!("Detected pause status change but will ignore it");
                        }
                    } else {
                        debug!("Unrecognized property name {}", name);
                    }
                }
                IpcEvent::PlaybackRestart => {
                    if self.app_flags.seeking {
                        debug!("Skipping playback restart due to previous seeking");
                        self.app_flags.seeking = false;
                        return;
                    }
                    if self.player.player_status == PlayerStatus::Buffering {
                        debug!("Player was buffering, setting to playing status");
                        self.set_player_to_playing();
                    } else if self.player.player_status == PlayerStatus::Playing {
                        debug!("Player was playing, setting to buffering status");
                        self.set_player_to_buffering();
                    }
                }
                IpcEvent::Eof(reason) => {
                    if reason == "eof" {
                        match self.player_data.loop_status {
                            AppLoopStatus::None => {
                                if self.queue_has_next() {
                                    self.go_next_queue();
                                    self.play_current(true);
                                }
                            }
                            AppLoopStatus::Track => {
                                warn!("Got eof while in loop, should not happen has mpv should have seeked to beginning.");
                                self.play_current(false);
                            }
                            AppLoopStatus::Playlist => {
                                if self.queue_has_next() {
                                    self.go_next_queue();
                                    self.play_current(true);
                                } else {
                                    debug!("Looping to first element in playlist");
                                    self.go_first_in_queue();
                                    self.play_current(false);
                                }
                            }
                        }
                    }
                }
                IpcEvent::Seek => {
                    self.app_flags.seeking = true;
                    let playback_time = self.player.get_playback_time();
                    debug!("Got {} as playback time", playback_time);
                    if playback_time != -1.0 {
                        self.ticks_during_playing_state = (playback_time * 4.0).floor() as usize;
                        if playback_time <= 1.0
                            && self.player_data.loop_status == AppLoopStatus::Track
                        {
                            // This means mpv forced the seek to 0 due to loop
                            debug!(
                                "Restarting song due to loop {}",
                                self.player_data.loop_status.as_str()
                            );
                            self.player_data.next_is_in_player_queue = true;
                            self.play_current(true);
                        }
                    }
                }
                IpcEvent::Idle => {
                    if self.player.player_status == PlayerStatus::Playing && !self.queue_has_next()
                    {
                        self.event_sender
                            .as_ref()
                            .unwrap()
                            .send(Dbus(Stop))
                            .unwrap();
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

    fn set_player_to_playing(&mut self) {
        if self.app_flags.was_paused {
            debug!("Player was paused, resuming playback");
            self.player.restore_player();
            self.app_flags.was_paused = false;
        }
        self.player.player_status = PlayerStatus::Playing;
        self.event_sender
            .as_ref()
            .unwrap()
            .send(Dbus(Playing))
            .unwrap();
        if !self.app_focused {
            self.event_sender
                .as_ref()
                .unwrap()
                .send(Draw(true))
                .unwrap();
        }
        if self.player_data.now_playing.duration == "0" {
            warn!("Duration was 0, trying to get duration from mpv!");
            let new_duration = self.player.get_duration();
            let new_duration_string = new_duration.trunc().to_string();
            debug!("New duration: {}", new_duration_string);
            if new_duration_string != "0" {
                self.database
                    .get_song_mut(self.player_data.now_playing.id.as_str())
                    .set_duration(new_duration_string.clone());
                self.player_data.now_playing.duration = new_duration_string;
                debug!("Modified song duration!");
            }
        }
    }

    fn set_player_to_buffering(&mut self) {
        self.player.player_status = PlayerStatus::Buffering;
        self.event_sender
            .as_ref()
            .unwrap()
            .send(Dbus(Paused))
            .unwrap();
        if !self.app_focused {
            self.event_sender
                .as_ref()
                .unwrap()
                .send(Draw(true))
                .unwrap();
        }
    }

    pub fn queue_has_next(&self) -> bool {
        if self.player_data.queue.len() <= 1 {
            false
        } else {
            self.player_data.index_in_queue < self.player_data.queue_order.len() - 1
        }
    }

    fn queue_has_previous(&self) -> bool {
        if self.player_data.queue.len() <= 1 {
            false
        } else {
            self.player_data.index_in_queue > 0
        }
    }

    pub fn get_playback_time(&self) -> usize {
        self.ticks_during_playing_state / 4
    }

    pub fn set_playback_time(&mut self, new_position_micros: i64) {
        let duration_micros = self
            .player_data
            .now_playing
            .duration
            .parse::<i64>()
            .unwrap()
            * 1000000;
        let new_position: f64 = if new_position_micros < 0 {
            0.0
        } else if new_position_micros > duration_micros {
            duration_micros as f64
        } else {
            new_position_micros as f64
        };
        let playback_percentage = ((new_position / duration_micros as f64) * 100.0).round();
        let percentage_string = format!("{}", playback_percentage as i64);
        debug!("Setting playing percentage: {}", percentage_string);
        self.player
            .set_playback_percentage(percentage_string.as_str());
        self.ticks_during_playing_state = (new_position / 1000000.0) as usize * 4;
    }

    fn go_next_queue(&mut self) {
        self.player_data.index_in_queue += 1;
        self.resolve_next_queue();
    }

    fn go_previous_queue(&mut self) {
        self.player_data.index_in_queue -= 1;
        self.resolve_next_queue();
    }

    fn go_first_in_queue(&mut self) {
        self.player_data.index_in_queue = 0;
        self.resolve_next_queue();
    }

    fn resolve_next_queue(&mut self) {
        let next_index = self
            .player_data
            .queue_order
            .get(self.player_data.index_in_queue)
            .unwrap();
        self.change_current_playing_to(
            self.player_data
                .queue
                .get(*next_index)
                .unwrap()
                .clone()
                .as_str(),
        );
        self.update_queue_data();
    }

    pub fn set_loop_mode(&mut self, loop_mode: &str) -> AppResult<()> {
        match loop_mode {
            "Track" => {
                debug!("Track loop");
                self.player_data.loop_status = AppLoopStatus::Track;
                self.player.set_loop_mode("inf");
            }
            "Playlist" => {
                debug!("Playlist loop");
                self.player_data.loop_status = AppLoopStatus::Playlist;
                self.player.set_loop_mode("no");
            }
            "None" => {
                debug!("None loop");
                self.player_data.loop_status = AppLoopStatus::None;
                self.player.set_loop_mode("no");
            }
            _ => {
                warn!("Loop setting unrecognized {}", loop_mode);
            }
        }
        self.player_data.next_is_in_player_queue = false;
        Ok(())
    }

    pub fn play_queue_song(&mut self) -> AppResult<()> {
        self.change_current_playing_to(
            self.player_data
                .queue
                .get(self.list_states.queue_list_state.selected().unwrap())
                .unwrap()
                .clone()
                .as_str(),
        );
        debug!(
            "Selected: {}, queue_order: {:?}",
            self.list_states.queue_list_state.selected().unwrap(),
            self.player_data.queue_order
        );
        self.player_data.index_in_queue = self.list_states.queue_list_state.selected().unwrap();
        if self.player_data.random_playback {
            self.shuffle_queue_order_starting_at_index(self.player_data.index_in_queue);
            debug!(
                "queue_order after shuffling: {:?}",
                self.player_data.queue_order
            );
            self.player_data.index_in_queue = 0;
        }
        self.play_current(false);
        self.update_queue_data();
        Ok(())
    }

    pub fn clear_queue(&mut self) -> AppResult<()> {
        self.player_data.queue.clear();
        self.player_data.queue_order.clear();
        self.player_data.now_playing.id.clear();
        self.player_data.index_in_queue = 0;
        self.event_sender
            .as_ref()
            .unwrap()
            .send(Dbus(Clear))
            .unwrap();
        Ok(())
    }

    pub fn delete_song_from_queue(&mut self) -> AppResult<()> {
        if self.player_data.queue.is_empty() {
            return Ok(());
        }

        let selected_index = if self.app_config.reorder_random_queue && self.player_data.random_playback {
                *self.player_data.queue_order
                    .get(self.list_states.queue_list_state.selected().unwrap())
                    .unwrap()
        } else {
            self.list_states.queue_list_state.selected().unwrap()
        };
        let deleted_song_id = self.player_data.queue[selected_index].clone();
        let is_selected_being_played = deleted_song_id == self.player_data.now_playing.id;
        let was_playing = self.player.player_status == PlayerStatus::Playing;
        
        self.player_data.queue.remove(selected_index);
        
        if let Some(pos) = self.player_data.queue_order.iter().position(|&x| x == selected_index) {
            self.player_data.queue_order.remove(pos);
            
            if is_selected_being_played {
                self.stop_playback();
                self.player_data.now_playing.id.clear();
                self.player_data.next_is_in_player_queue = false;
                
                if !self.player_data.queue.is_empty() {
                    if self.player_data.index_in_queue >= self.player_data.queue_order.len() {
                        self.player_data.index_in_queue = 0;
                    }
                } else {
                    self.player_data.index_in_queue = 0;
                }
            } else if pos < self.player_data.index_in_queue {
                self.player_data.index_in_queue -= 1;
            }
        }
        
        for order_index in self.player_data.queue_order.iter_mut() {
            if *order_index > selected_index {
                *order_index -= 1;
            }
        }
        
        self.update_queue_data();
        
        if is_selected_being_played && !self.player_data.queue.is_empty() {
            let next_song_index = self.player_data.queue_order[self.player_data.index_in_queue];
            let next_song_id = self.player_data.queue[next_song_index].clone();
            self.change_current_playing_to(&next_song_id);
            if was_playing {
                self.play_current(false);
            }
        }
        
        if self.player_data.queue.is_empty() {
            self.list_states.queue_list_state.select_first();
        } else if selected_index >= self.player_data.queue.len() {
            self.list_states.queue_list_state.select(Some(self.player_data.queue.len() - 1));
        }
        
        Ok(())
    }

    pub fn try_play_current(&mut self) -> bool {
        if !self.player_data.now_playing.id.is_empty() {
            return if self.player.player_status == PlayerStatus::Paused
                || self.player.player_status == PlayerStatus::Buffering
            {
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
        if !self.player_data.now_playing.id.is_empty()
            && self.player.player_status == PlayerStatus::Playing
        {
            self.toggle_playing_status().unwrap();
            return true;
        }
        false
    }

    pub fn stop_playback(&mut self) {
        self.player.stop();
        self.player.restore_player();
        self.ticks_during_playing_state = 0;
        self.player.player_status = PlayerStatus::Stopped;
    }

    fn play_current(&mut self, check_next_song: bool) {
        if !check_next_song || !self.player_data.next_is_in_player_queue {
            debug!(
                "Adding song {} to player queue",
                self.player_data.now_playing.id
            );
            self.player.play_song(
                self.server
                    .get_song_url(self.player_data.now_playing.id.clone())
                    .as_str(),
            );
            if self.player.player_status == PlayerStatus::Paused {
                self.app_flags.was_paused = true;
            }
            self.player.player_status = PlayerStatus::LoadingFile;
            self.event_sender
                .as_ref()
                .unwrap()
                .send(Dbus(Paused))
                .unwrap();
        }
        self.player_data.next_is_in_player_queue = false;
        self.app_flags.is_current_song_scrobbled = false;
        self.ticks_during_playing_state = 0;
        self.center_queue_cursor().unwrap();
        self.event_sender
            .as_ref()
            .unwrap()
            .send(Dbus(Metadata))
            .unwrap();

        if !self.app_focused {
            self.event_sender
                .as_ref()
                .unwrap()
                .send(Draw(true))
                .unwrap();
        }
    }

    fn shuffle_queue_order_starting_at_index(&mut self, index: usize) {
        let mut shuffled_vector = Vec::with_capacity(self.player_data.queue.len());
        self.player_data.queue_order.swap_remove(index);
        shuffled_vector.push(index);

        let mut rng = rng();
        self.player_data.queue_order.shuffle(&mut rng);

        shuffled_vector.append(&mut self.player_data.queue_order);
        self.player_data.queue_order = shuffled_vector;
    }

    fn change_current_playing_to(&mut self, new_id: &str) {
        self.player_data.now_playing.id = String::from(new_id);
        self.player_data.now_playing.duration =
            String::from(self.database.get_song(new_id).duration());
    }

    pub async fn set_event_handler(&mut self, sender: UnboundedSender<Event>) -> AppResult<()> {
        self.event_sender = Some(sender);
        Ok(())
    }

    pub fn get_metadata_for_current_song(&mut self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        let song = self
            .database
            .get_song(self.player_data.now_playing.id.as_str());
        metadata.insert("title".to_string(), song.title().to_string());
        metadata.insert("album".to_string(), song.album().to_string());
        metadata.insert("artist".to_string(), song.artist().to_string());
        metadata.insert(
            "id".to_string(),
            song.id().chars().filter(|c| c.is_alphanumeric()).collect(),
        );
        metadata.insert("length".to_string(), song.duration().to_string());
        metadata.insert(
            "cover".to_string(),
            self.server.get_song_art_url(song.id().to_string()),
        );

        metadata
    }

    pub fn cycle_pane(&mut self) -> AppResult<()> {
        if self.current_popup == Popup::None {
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
        } else {
            match self.current_popup {
                Popup::GlobalSearch => match self.global_search_pane {
                    FourPaneGrid::TopLeft => self.global_search_pane = FourPaneGrid::TopRight,
                    FourPaneGrid::TopRight => self.global_search_pane = FourPaneGrid::BottomLeft,
                    FourPaneGrid::BottomLeft => self.global_search_pane = FourPaneGrid::BottomRight,
                    FourPaneGrid::BottomRight => self.global_search_pane = FourPaneGrid::TopLeft,
                },
                _ => {}
            }
        }

        Ok(())
    }

    pub fn try_go_up_pane(&mut self) -> AppResult<()> {
        if self.current_popup == Popup::None {
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
        } else {
            match self.current_popup {
                Popup::GlobalSearch => match self.global_search_pane {
                    FourPaneGrid::BottomLeft => self.global_search_pane = FourPaneGrid::TopLeft,
                    FourPaneGrid::BottomRight => self.global_search_pane = FourPaneGrid::TopRight,
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(())
    }

    pub fn try_go_down_pane(&mut self) -> AppResult<()> {
        if self.current_popup == Popup::None {
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
        } else {
            match self.current_popup {
                Popup::GlobalSearch => match self.global_search_pane {
                    FourPaneGrid::TopLeft => self.global_search_pane = FourPaneGrid::BottomLeft,
                    FourPaneGrid::TopRight => self.global_search_pane = FourPaneGrid::BottomRight,
                    _ => {}
                },
                _ => {}
            }
        }
        Ok(())
    }

    pub fn try_go_left_pane(&mut self) -> AppResult<()> {
        if self.current_popup == Popup::None {
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
                CurrentScreen::Albums => {
                    self.album_pane = TwoPaneVertical::Left;
                }
                CurrentScreen::Playlists => {
                    self.playlist_pane = TwoPaneVertical::Left;
                }
                CurrentScreen::Artists => {
                    self.artist_pane = TwoPaneVertical::Left;
                }
                _ => {}
            }
        } else {
            match self.current_popup {
                Popup::GlobalSearch => match self.global_search_pane {
                    FourPaneGrid::TopRight => self.global_search_pane = FourPaneGrid::TopLeft,
                    FourPaneGrid::BottomRight => self.global_search_pane = FourPaneGrid::BottomLeft,
                    _ => {}
                },
                _ => {}
            }
        }

        Ok(())
    }
    pub fn try_go_right_pane(&mut self) -> AppResult<()> {
        if self.current_popup == Popup::None {
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
                CurrentScreen::Albums => {
                    self.album_pane = TwoPaneVertical::Right;
                }
                CurrentScreen::Playlists => {
                    self.playlist_pane = TwoPaneVertical::Right;
                }
                CurrentScreen::Artists => {
                    self.artist_pane = TwoPaneVertical::Right;
                }
                _ => {}
            }
        } else {
            match self.current_popup {
                Popup::GlobalSearch => match self.global_search_pane {
                    FourPaneGrid::BottomLeft => self.global_search_pane = FourPaneGrid::BottomRight,
                    FourPaneGrid::TopLeft => self.global_search_pane = FourPaneGrid::TopRight,
                    _ => {}
                },
                _ => {}
            }
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
        self.album_filters.genre_filter =
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

    pub fn toggle_favorite_genre(&mut self) -> AppResult<()> {
        // We subtract 1 from the list state because of the first item "Any"
        if self.list_states.popup_genre_list_state.selected().unwrap() == 0 {
            debug!("Won't set 'Any' as favorite!");
            return Ok(());
        }
        let selected_genre = self
            .database
            .genres()
            .get(self.list_states.popup_genre_list_state.selected().unwrap() - 1)
            .unwrap();
        if let Some(position) = self
            .database
            .favorite_genres()
            .iter()
            .position(|x| x == selected_genre)
        {
            debug!(
                "Removing favorite genre {} at position {}",
                selected_genre, position
            );
            self.database.remove_favorite_genre(position);
        } else if self.database.favorite_genres().len() <= 9 {
            debug!(
                "Genre {} was not found, adding to favorites",
                selected_genre
            );
            self.database.push_favorite_genre(selected_genre.clone());
        } else {
            warn!(
                "Genre {} was not added to favorites because the list is full",
                selected_genre
            );
        }
        Ok(())
    }
    pub fn set_favorite_genre_filter(&mut self, position: usize) -> AppResult<()> {
        // Position in popup are labeled as 1-9
        self.album_filters.genre_filter = self
            .database
            .favorite_genres()
            .get(position - 1)
            .unwrap()
            .clone();
        Ok(())
    }

    pub fn center_queue_cursor(&mut self) -> AppResult<()> {
        let index = if self.app_config.reorder_random_queue {
            self.player_data.index_in_queue
        } else {
            self.player_data.queue_order[self.player_data.index_in_queue]
        };
        self.list_states.queue_list_state.select(Some(index));
        Ok(())
    }

    pub fn process_filtered_album_list(&mut self) -> AppResult<()> {
        let list = if self.album_sorting_mode == SortMode::Frequent {
            self.database.most_listened_albums()
        } else {
            self.database.alphabetical_list_albums()
        };

        let new_filtered_list: Vec<String> = if self.album_filters.genre_filter != "any"
            && self.album_filters.year_from_filter.is_empty()
        {
            list.iter()
                .filter_map(|album_id| {
                    if self
                        .database
                        .album_contains_genre(album_id, &self.album_filters.genre_filter)
                    {
                        Some(album_id.clone())
                    } else {
                        None
                    }
                })
                .collect()
        } else if self.album_filters.genre_filter != "any"
            && !self.album_filters.year_from_filter.is_empty()
            && self.album_filters.year_to_filter.is_empty()
        {
            list.iter()
                .filter_map(|album_id| {
                    if self
                        .database
                        .album_contains_genre(album_id, &self.album_filters.genre_filter)
                        && self.database.get_album(album_id).year()
                            == self.album_filters.year_from_filter
                    {
                        Some(album_id.clone())
                    } else {
                        None
                    }
                })
                .collect()
        } else if self.album_filters.genre_filter != "any"
            && !self.album_filters.year_from_filter.is_empty()
            && !self.album_filters.year_to_filter.is_empty()
        {
            list.iter()
                .filter_map(|album_id| {
                    if self
                        .database
                        .album_contains_genre(album_id, &self.album_filters.genre_filter)
                        && self
                            .database
                            .get_album(album_id)
                            .year()
                            .parse::<i32>()
                            .unwrap()
                            >= self.album_filters.year_from_filter.parse().unwrap()
                        && self
                            .database
                            .get_album(album_id)
                            .year()
                            .parse::<i32>()
                            .unwrap()
                            <= self.album_filters.year_to_filter.parse().unwrap()
                    {
                        Some(album_id.clone())
                    } else {
                        None
                    }
                })
                .collect()
        } else if self.album_filters.genre_filter == "any"
            && !self.album_filters.year_from_filter.is_empty()
            && self.album_filters.year_to_filter.is_empty()
        {
            list.iter()
                .filter_map(|album_id| {
                    if self.database.get_album(album_id).year()
                        == self.album_filters.year_from_filter
                    {
                        Some(album_id.clone())
                    } else {
                        None
                    }
                })
                .collect()
        } else if self.album_filters.genre_filter == "any"
            && !self.album_filters.year_from_filter.is_empty()
            && !self.album_filters.year_to_filter.is_empty()
        {
            list.iter()
                .filter_map(|album_id| {
                    if !self.database.get_album(album_id).year().is_empty()
                        && self
                            .database
                            .get_album(album_id)
                            .year()
                            .parse::<i32>()
                            .unwrap()
                            >= self.album_filters.year_from_filter.parse().unwrap()
                        && self
                            .database
                            .get_album(album_id)
                            .year()
                            .parse::<i32>()
                            .unwrap()
                            <= self.album_filters.year_to_filter.parse().unwrap()
                    {
                        Some(album_id.clone())
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            list.clone()
        };

        self.database.set_filtered_albums(new_filtered_list);
        Ok(())
    }

    pub fn toggle_sort_order(&mut self) -> AppResult<()> {
        self.database.filtered_albums_mut().reverse();
        self.database.alphabetical_list_albums_mut().reverse();
        self.database.most_listened_albums_mut().reverse();
        if self.album_sorting_direction == SortOrder::Descending {
            self.album_sorting_direction = SortOrder::Ascending;
        } else {
            self.album_sorting_direction = SortOrder::Descending;
        }
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
                Popup::GlobalSearch => match self.global_search_pane {
                    FourPaneGrid::TopLeft => &mut self.list_states.global_search_songs,
                    FourPaneGrid::TopRight => &mut self.list_states.global_search_albums,
                    FourPaneGrid::BottomLeft => &mut self.list_states.global_search_playlists,
                    FourPaneGrid::BottomRight => &mut self.list_states.global_search_artists,
                },
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
                        .take(self.app_config.list_size)
                        .map(|album_id| self.database.get_album(album_id).name().to_string())
                        .collect::<Vec<String>>(),
                    HomePane::Bottom => self
                        .database
                        .most_listened_albums()
                        .iter()
                        .take(self.app_config.list_size)
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
                        .take(self.app_config.list_size)
                        .map(|album_id| self.database.get_album(album_id).name().to_string())
                        .collect::<Vec<String>>(),
                    HomePane::TopRight => self
                        .database
                        .recently_added_albums()
                        .iter()
                        .take(self.app_config.list_size)
                        .map(|album_id| self.database.get_album(album_id).name().to_string())
                        .collect::<Vec<String>>(),
                    HomePane::BottomLeft => self
                        .database
                        .most_listened_albums()
                        .iter()
                        .take(self.app_config.list_size)
                        .map(|album_id| self.database.get_album(album_id).name().to_string())
                        .collect::<Vec<String>>(),
                    HomePane::BottomRight => self
                        .database
                        .most_listened_tracks()
                        .iter()
                        .take(self.app_config.list_size)
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
                .player_data
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
            if album_id != DEFAULT_ALBUM {
                info!("Album {} not found in server, deleting", album_id);
                // First we get all artists related to this album (through the songs)
                let artist_ids: HashSet<_> = self
                    .database
                    .songs()
                    .iter()
                    .filter(|(_, song)| song.album_id() == album_id)
                    .map(|(_, song)| song.artist_id().to_string())
                    .collect();
                // Then we remove the album
                self.database.remove_album(album_id.as_str());
                // Finally we update the artists
                for artist_id in artist_ids {
                    self.database.update_artist(artist_id.as_str());
                }
            }
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

    pub fn validate_year_filters(&mut self) -> AppResult<()> {
        let year1: i32 = self
            .album_filters
            .year_from_filter_new
            .parse()
            .map_err(|_| format!("Invalid year: {}", self.album_filters.year_from_filter_new))?;
        let year2: i32 = self
            .album_filters
            .year_to_filter_new
            .parse()
            .map_err(|_| format!("Invalid year: {}", self.album_filters.year_to_filter_new))?;

        if year2 < year1 {
            warn!(
                "Could not set filters because the 'to' = {} cannot be before the 'from' = {}",
                year2, year1
            );
            self.album_filters.filter_message = "Cannot set the 'to' before the 'from'".to_string();
        } else if year1 == year2 {
            info!("'From' and 'to' year filters are equal, ignoring 'to'");
            self.album_filters.year_to_filter.clear();
            self.album_filters.year_to_filter_new.clear();
            self.album_filters.filter_message.clear();
        } else {
            self.album_filters.filter_message.clear();
        }

        Ok(())
    }

    pub fn get_selected_album_for_update(&mut self) -> AppResult<()> {
        let selected_album_id = match self.current_screen {
            CurrentScreen::Home => match self.home_pane {
                HomePane::TopLeft => self
                    .database
                    .recent_albums()
                    .get(self.list_states.home_tab_top_left.selected().unwrap())
                    .cloned(),
                HomePane::TopRight => self
                    .database
                    .recently_added_albums()
                    .get(self.list_states.home_tab_top_right.selected().unwrap())
                    .cloned(),
                HomePane::BottomLeft => self
                    .database
                    .most_listened_albums()
                    .get(self.list_states.home_tab_bottom_left.selected().unwrap())
                    .cloned(),
                HomePane::BottomRight => Some(
                    self.database
                        .get_song(
                            self.database
                                .most_listened_tracks()
                                .get(self.list_states.home_tab_bottom_right.selected().unwrap())
                                .unwrap(),
                        )
                        .album_id()
                        .to_string(),
                ),
                _ => {
                    panic!("Should not happen")
                }
            },
            CurrentScreen::Albums => self
                .database
                .filtered_albums()
                .get(self.list_states.album_state.selected().unwrap())
                .cloned(),
            CurrentScreen::Playlists => {
                if self.database
                    .alphabetical_playlists()
                    .get(self.list_states.playlist_state.selected().unwrap()).is_none() {
                    warn!("Not possible to get playlist to update its albums");
                    Some(DEFAULT_ALBUM.to_string())
                } else {
                    Some(
                        self.database
                            .get_song(
                                self.database
                                    .get_playlist(
                                        self.database
                                            .alphabetical_playlists()
                                            .get(self.list_states.playlist_state.selected().unwrap())
                                            .unwrap(),
                                    )
                                    .song_list()
                                    .get(
                                        self.list_states
                                            .playlist_selected_state
                                            .selected()
                                            .unwrap_or(0),
                                    )
                                    .unwrap_or(&DEFAULT_ALBUM.to_string())
                            )
                            .album_id()
                            .to_string(),
                    )
                }
            },
            CurrentScreen::Artists => {
                let albums = self
                    .database
                    .get_artist(
                        self.database
                            .alphabetical_artists()
                            .get(self.list_states.artist_state.selected().unwrap())
                            .unwrap(),
                    )
                    .albums();
                let (selected_album_index, _offset) =
                    self.get_selected_album_index_in_artist_view(albums);
                albums.get(selected_album_index).cloned()
            }
            CurrentScreen::Queue => self
                .list_states
                .queue_list_state
                .selected()
                .and_then(|i| self.player_data.queue.get(i))
                .map(|song_id| self.database.get_song(song_id).album_id().to_string()),
        };

        match selected_album_id {
            None => {
                warn!("No album selected");
            }
            Some(album_id) => {
                if album_id == DEFAULT_ALBUM {
                    warn!("Not possible to update this album");
                } else {
                    self.selected_album_id_to_update = album_id.clone();
                }
            }
        }

        Ok(())
    }

    pub fn set_album_in_list_to_current_playing(&mut self) -> AppResult<()> {
        let selected_queue_song_id = if self.app_config.reorder_random_queue && self.player_data.random_playback {
            let selected_song_in_queue = 
                self.player_data.queue_order
                    .get(self.list_states.queue_list_state.selected().unwrap())
                    .unwrap();
            self.player_data.queue.get(*selected_song_in_queue).unwrap()
        } else {
            self.player_data.queue[self.list_states.queue_list_state.selected().unwrap()].as_str()
        };
        let album_id = self
            .database
            .get_song(selected_queue_song_id)
            .album_id()
            .to_string();
        self.set_album_index_for_album_id(album_id.as_str())?;
        Ok(())
    }

    fn set_album_index_for_album_id(&mut self, album_id: &str) -> AppResult<()>{
        let index = self
            .database
            .filtered_albums()
            .iter()
            .position(|album| album == album_id);
        match index {
            Some(index) => {
                self.list_states.album_state.select(Some(index));
                Ok(())
            }
            None => {
                warn!("Could not find the index for the album id: {}", album_id);
                Err(format!("Could not find the album ({}) in the album list!", album_id).into())
            }
        }
    }

    pub fn set_artist_in_list_to_current_playing(&mut self) -> AppResult<()> {
        let selected_queue_song_id = if self.app_config.reorder_random_queue && self.player_data.random_playback {
            let selected_song_in_queue =
                self.player_data.queue_order
                    .get(self.list_states.queue_list_state.selected().unwrap())
                    .unwrap();
            self.player_data.queue.get(*selected_song_in_queue).unwrap()
        } else {
            self.player_data.queue[self.list_states.queue_list_state.selected().unwrap()].as_str()
        };
        let selected_queue_song = self.database.get_song(selected_queue_song_id);
        let index = self
            .database
            .alphabetical_artists()
            .iter()
            .position(|artist| artist == selected_queue_song.artist_id());
        match index {
            Some(index) => {
                self.list_states.artist_state.select(Some(index));
            }
            None => {
                warn!("Could not find the artist ({}) of the selected song ({}) in the artist list!", selected_queue_song.artist(), selected_queue_song.title());
                return Err(format!("Could not find the artist ({}) of the selected song ({}) in the artist list!", selected_queue_song.artist(), selected_queue_song.title()).into());
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
                self.event_sender
                    .as_ref()
                    .unwrap()
                    .send(Draw(true))
                    .unwrap();
            }
        }

        // We will prioritize fetching the alphabetical album list to ensure we have all albums
        // before anything else
        for i in 0..pending_operations_number {
            let operation = &mut self.server.operations[i];
            if operation.error() {
                error!("Operation {:?} failed", operation.operation_id());
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
                            match Parser::parse_playlist_list(operation.result().to_string(), self.app_config.parser_type) {
                                Ok(playlist_list) => playlist_list,
                                Err(e) => {
                                    warn!(
                                        "Could not parse playlist list result: {}",
                                        operation.result()
                                    );
                                    warn!("{}", e);
                                    operation.set_processed(true);
                                    continue;
                                }
                            };
                        operation.set_processed(true);
                        for playlist in playlist_list {
                            if self.database.contains_playlist(playlist.id()) && !force_update {
                                debug!("Playlist {} already in database", playlist.name());
                                match is_date_before(
                                    self.database.get_playlist(playlist.id()).modified_on(),
                                    playlist.modified_on(),
                                ) {
                                    Ok(result) => {
                                        if result {
                                            debug!(
                                                "Playlist {} has a newer modified date in server",
                                                playlist.name()
                                            );
                                            if self.database.get_playlist(playlist.id()).modified()
                                            {
                                                debug!("Playlist {} has been modified locally, will not pull from server!", playlist.name());
                                            } else {
                                                debug!(
                                                    "Fetching server version of playlist {}",
                                                    playlist.name()
                                                );
                                                self.server.get_playlist_async(playlist.id());
                                                self.database.remove_playlist(playlist.id());
                                                self.database.insert_playlist(
                                                    playlist.id().to_string(),
                                                    playlist,
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error! {"Error while comparing dates for playlist {}: {}", playlist.name(), e}
                                    }
                                }
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
                                self.database.remove_playlist(playlist.id());
                                self.database
                                    .insert_playlist(playlist.id().to_string(), playlist);
                            }
                        }
                    }
                    Operation::GetPlaylist(id) => {
                        if self.app_flags.updating_albums
                            || self.app_flags.updating_alphabetical_albums
                        {
                            continue;
                        }
                        let playlist_songs =
                            match Parser::parse_playlist(operation.result().to_string(), self.app_config.parser_type) {
                                Ok(playlist_songs) => playlist_songs,
                                Err(e) => {
                                    warn!(
                                        "Could not parse playlist result: {}",
                                        operation.result()
                                    );
                                    warn!("{}", e);
                                    operation.set_processed(true);
                                    continue;
                                }
                            };
                        self.database.set_playlist_songs(id, playlist_songs);
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
                        match Parser::parse_playlist_id(operation.result().to_string(), self.app_config.parser_type) {
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
                            match Parser::parse_album_list_simple(operation.result().to_string(), self.app_config.album_list_api_namespace.as_str(), self.app_config.parser_type) {
                                Ok(album_list) => album_list,
                                Err(e) => {
                                    warn!("Could not parse album list: {}", operation.result());
                                    warn!("{}", e);
                                    continue;
                                }
                            };
                        if !album_list.is_empty() {
                            let new_offset = offset + album_list.len();
                            for album_id in &album_list {
                                if self.database.contains_album(album_id.as_str()) && !force_update
                                {
                                    debug!("Album {} already in database", album_id);
                                } else if !self.database.contains_album(album_id.as_str()) {
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
                        let album;
                        let songs;
                        let artist;
                        match Parser::parse_album(operation.result().to_string(), self.app_config.parser_type) {
                            Ok(parsed_items) => {
                                album = parsed_items.0;
                                songs = parsed_items.1;
                                artist = parsed_items.2;
                            }
                            Err(e) => {
                                warn!("Could not parse album result: {}", operation.result());
                                warn!("{}", e);
                                self.albums_being_updated -= 1;
                                operation.set_processed(true);
                                if self.albums_being_updated == 0
                                    && !self.app_flags.updating_alphabetical_albums
                                {
                                    self.app_flags.updating_albums = false;
                                }
                                continue;
                            }
                        }

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
                            debug!(
                                "Artist {} already had album {}, forcing update",
                                artist.name(),
                                album_id
                            );
                            self.database
                                .get_artist_mut(artist.id())
                                .albums_mut()
                                .retain(|id| id != album_id);
                            self.database
                                .get_artist_mut(artist.id())
                                .insert_album(album_id.clone(), album_genres);
                            self.database.update_artist(artist.id());
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
                            match Parser::parse_album_list_simple(operation.result().to_string(), self.app_config.album_list_api_namespace.as_str(), self.app_config.parser_type) {
                                Ok(album_list) => album_list,
                                Err(e) => {
                                    warn!("Could not parse album list result: {}", operation.result());
                                    warn!("{}", e);
                                    continue;
                                }
                            };
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
                            match Parser::parse_album_list_simple(operation.result().to_string(), self.app_config.album_list_api_namespace.as_str(), self.app_config.parser_type) {
                                Ok(album_list) => album_list,
                                Err(e) => {
                                    warn!("Could not parse album list result: {}", operation.result());
                                    warn!("{}", e);
                                    continue;
                                }
                            };
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
                        operation.set_processed(true);
                        let mut genres =
                            match Parser::parse_genres_list(operation.result().to_string(), self.app_config.parser_type) {
                                Ok(genres) => genres,
                                Err(e) => {
                                    warn!("Could not parse genres result: {}", operation.result());
                                    warn!("{}", e);
                                    continue;
                                }
                            };
                        genres.sort();
                        self.database.set_genres(genres);
                    }
                    Operation::GetAlbumListRecentlyAdded() => {
                        if self.app_flags.updating_albums {
                            continue;
                        }
                        operation.set_processed(true);
                        let album_list =
                            match Parser::parse_album_list_simple(operation.result().to_string(), self.app_config.album_list_api_namespace.as_str(), self.app_config.parser_type) {
                                Ok(album_list) => album_list,
                                Err(e) => {
                                    warn!("Could not parse album list result: {}", operation.result());
                                    warn!("{}", e);
                                    continue;
                                }
                            };
                        self.database.set_recently_added_albums(album_list);
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
        // We subtract 1 to number of albums to account for default album
        if self.database.get_number_of_albums() - 1 > self.database.alphabetical_list_albums().len()
        {
            debug!("Number of albums in database ({}) is greater than alphabetical list ({}), albums have been deleted!",
                                    self.database.get_number_of_albums() - 1, self.database.alphabetical_list_albums().len());
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

    pub fn update_selected_album(&mut self) -> AppResult<()> {
        self.app_flags.updating_albums = true;
        self.albums_being_updated += 1;
        self.server
            .get_album_async(self.selected_album_id_to_update.clone());
        Ok(())
    }

    pub fn get_global_search_results(&mut self) {
        let search_str = self.search_data.global_search_string.to_lowercase();

        let song_results: Vec<String> = self
            .database
            .songs()
            .iter()
            .filter(|(_id, song)| song.title().to_lowercase().contains(&search_str))
            .map(|(id, _song)| id.clone())
            .collect();

        let album_results: Vec<String> = self
            .database
            .albums()
            .iter()
            .filter(|(_, album)| album.name().to_lowercase().contains(&search_str))
            .map(|(id, _)| id.clone())
            .collect();

        let artist_results: Vec<String> = self
            .database
            .artists()
            .iter()
            .filter(|(_, artist)| artist.name().to_lowercase().contains(&search_str))
            .map(|(id, _)| id.clone())
            .collect();

        let playlist_results: Vec<String> = self
            .database
            .playlists()
            .iter()
            .filter(|(_, playlist)| playlist.name().to_lowercase().contains(&search_str))
            .map(|(id, _)| id.clone())
            .collect();

        self.search_data.global_search_song_results = song_results;
        self.search_data.global_search_albums_results = album_results;
        self.search_data.global_search_artists_results = artist_results;
        self.search_data.global_search_playlists_results = playlist_results;
    }
}
fn sort_songs_by_play_count(songs: &HashMap<String, Song>) -> Vec<String> {
    let mut song_vector: Vec<_> = songs
        .iter()
        .filter(|(id,_)| *id != DEFAULT_SONG)
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

fn update_queue_order_when_adding_next(player_data: &mut PlayerData, index: usize, value: usize) {
    if !player_data.random_playback {
        player_data
            .queue_order
            .push(player_data.queue.len() - 1);
    } else {
        for i in 0..player_data.queue_order.len() {
            if player_data.queue_order[i] >= index {
                player_data.queue_order[i] += 1;
            }
        }
        debug!("Inserting value {} at index {}", value, index);
        player_data.queue_order.insert(index, value);
    }
}


fn parse_color(string_color: &str) -> AppResult<Color> {
    if let Ok(color) = Color::from_str(string_color) {
        Ok(color)
    } else {
        Err(Box::from("Could not parse color"))
    }
}

fn is_date_before(date1: &str, date2: &str) -> AppResult<bool> {
    let format = "%m/%d/%y - %H:%M";

    // Parse the date strings into NaiveDateTime objects
    let parsed_date1 = NaiveDateTime::parse_from_str(date1, format)?;
    let parsed_date2 = NaiveDateTime::parse_from_str(date2, format)?;

    // Return true if the first date is before the second one
    Ok(parsed_date1 < parsed_date2)
}
