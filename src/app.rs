use std::collections::HashMap;
use std::error;

use config::Config;
use log::debug;
use rand::seq::SliceRandom;
use rand::thread_rng;
use ratatui::widgets::ListState;
use tokio::sync::mpsc::UnboundedSender;
use crate::event::DbusEvent::{Clear, Playing};
use crate::event::Event;
use crate::event::Event::Dbus;
use crate::music_database::MusicDatabase;
use crate::player::ipc::IpcEvent;
use crate::player::mpv::{Mpv, PlayerStatus};
use crate::server::Server;

/// Enum with applications screens
#[derive(Debug)]
pub enum CurrentScreen {
    Home,
    Albums,
    Playlists,
    Artists,
    Queue,
}

#[derive(Debug, Default)]
pub enum MediaType {
    #[default]
    Song,
    Album,
    Playlist,
}

/// Enum with applications screens
#[derive(Debug, PartialEq)]
pub enum Popup {
    ConnectionTest,
    AlbumInformation,
    AddTo,
    None,
}

/// Application result type.
pub type AppResult<T> = Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    pub current_screen: CurrentScreen,
    pub current_popup: Popup,
    pub previous_popup: Popup,
    pub server: Server,
    pub event_sender: Option<UnboundedSender<Event>>,
    pub database: MusicDatabase,
    pub home_recent_state: ListState,
    pub queue_list_state: ListState,
    pub popup_list_state: ListState,
    pub item_to_be_added: ItemToBeAdded,
    pub queue: Vec<String>,
    pub queue_order: Vec<usize>,
    pub now_playing: NowPlaying,
    pub player: Mpv,
    pub index_in_queue: usize,
    pub ticks_during_playing_state: usize,
    pub random_playback: bool,
    pub next_is_in_player_queue: bool
}

#[derive(Default, Debug)]
pub struct ItemToBeAdded {
    pub name: String,
    pub id: String,
    pub parent_id: String,
    pub media_type: MediaType,
}

#[derive(Default)]
pub struct NowPlaying {
    pub id: String,
    pub duration: String
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            current_screen: CurrentScreen::Home,
            current_popup: Popup::None,
            previous_popup: Popup::None,
            server: Server::new(),
            event_sender: None,
            database: MusicDatabase::new(),
            home_recent_state: ListState::default(),
            queue_list_state: ListState::default(),
            popup_list_state: ListState::default(),
            item_to_be_added: ItemToBeAdded::default(),
            queue: vec![],
            queue_order: vec![],
            now_playing: NowPlaying::default(),
            player: Mpv::default(),
            index_in_queue: 0,
            ticks_during_playing_state: 0,
            random_playback: false,
            next_is_in_player_queue: false
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
        self.process_player_events();
        if *self.player.player_status() == PlayerStatus::Playing {
            self.ticks_during_playing_state += 1;
        }
        if !self.next_is_in_player_queue && self.queue_has_next() && 
           (self.get_playback_time() + 10 > self.now_playing.duration.as_str().parse::<usize>().unwrap()) {
            let next_index = self.queue_order.get(self.index_in_queue + 1).unwrap();
            self.player.add_next_song_to_queue(self.server.get_song_url(self.queue.get(*next_index).unwrap().clone()).as_str());
            self.next_is_in_player_queue = true;
        }
    }

    /// Set running to false in order to quit the application.
    pub fn quit(&mut self) {
        self.player.quit_player();
        self.running = false;
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

    pub async fn populate_db(&mut self) -> AppResult<()> {
        self.database.set_recent_albums(self.server.get_recent_albums().await?);
        Ok(())
    }

    pub fn initialize_player_stream(&mut self) -> AppResult<()> {
        // TODO Try to capture connection error and retry, to give mpv time to initialize
        self.player.initialize();
        Ok(())
    }

    pub async fn poll_player_events(&mut self) -> AppResult<()> {
        self.player.poll_ipc_events().await;
        Ok(())
    }

    pub async fn get_current_album_information(&mut self) -> AppResult<()> {
        let selected_album_index = self.home_recent_state.selected().unwrap();
        let selected_album_id: String = self.database.recent_albums().get(selected_album_index).unwrap().id().to_string();

        if !self.database.contains_album(selected_album_id.as_str()) {
            populate_album_in_db(&mut self.server, &mut self.database, selected_album_id.as_str()).await?;
        }
        Ok(())
    }


    pub fn select_next_list(&mut self) -> AppResult<()> {
        self.home_recent_state.select_next();
        Ok(())
    }

    pub fn select_previous_list(&mut self) -> AppResult<()> {
        self.home_recent_state.select_previous();
        Ok(())
    }

    pub fn select_next_list_popup(&mut self) -> AppResult<()> {
        self.popup_list_state.select_next();
        Ok(())
    }

    pub fn select_previous_list_popup(&mut self) -> AppResult<()> {
        self.popup_list_state.select_previous();
        Ok(())
    }

    pub fn select_next_queue(&mut self) -> AppResult<()> {
        self.queue_list_state.select_next();
        Ok(())
    }

    pub fn select_previous_queue(&mut self) -> AppResult<()> {
        self.queue_list_state.select_previous();
        Ok(())
    }

    pub async fn add_queue_immediately(&mut self) -> AppResult<()> {
        self.index_in_queue = 0;
        match self.item_to_be_added.media_type {
            MediaType::Song => {
                self.queue.clear();
                self.queue_order.clear();
                self.queue.push(self.item_to_be_added.id.clone());
                self.queue_order.push(self.queue.len() - 1);
                self.change_current_playing_to(self.item_to_be_added.id.clone().as_str());
                self.play_current(false);
            }
            MediaType::Album => {
                self.queue.clear();
                self.queue_order.clear();
                if !self.database.contains_album(self.item_to_be_added.id.as_str()) {
                    populate_album_in_db(&mut self.server, &mut self.database, self.item_to_be_added.id.as_str()).await?;
                }
                let album = self.database.get_album(self.item_to_be_added.id.as_str());
                for  song in album.songs() {
                    self.queue.push(song.clone());
                }
                self.queue_order = (0..self.queue.len()).collect();
                if self.random_playback {self.shuffle_queue_order_starting_at_current_index()}
                self.change_current_playing_to(self.queue.first().unwrap().clone().as_str());
                self.play_current(false);
            }
            MediaType::Playlist => {}
        }
        Ok(())
    }

    pub fn add_queue_next(&mut self) -> AppResult<()> {
        match self.item_to_be_added.media_type {
            MediaType::Song => {
                let index = self.queue.iter().position(|x| x == &self.now_playing.id).unwrap();
                self.queue.insert(index + 1, self.item_to_be_added.id.clone());
                self.queue_order.push(self.queue.len() - 1);
            }
            MediaType::Album => {}
            MediaType::Playlist => {}
        }
        Ok(())
    }

    pub fn add_queue_later(&mut self) -> AppResult<()> {
        match self.item_to_be_added.media_type {
            MediaType::Song => {
                self.queue.push(self.item_to_be_added.id.clone());
                self.queue_order.push(self.queue.len() - 1);
            }
            MediaType::Album => {}
            MediaType::Playlist => {}
        }
        Ok(())
    }

    pub fn set_item_to_be_added(&mut self, media: MediaType) -> AppResult<()> {
        match media {
            MediaType::Song => {
                let selected_album_index = self.home_recent_state.selected().unwrap();
                let selected_album_id = self.database.recent_albums().get(selected_album_index).unwrap().id();
                let songs_ids = self.database.get_album(selected_album_id).songs();
                let song = self.database.get_song(songs_ids.get(self.popup_list_state.selected().unwrap()).unwrap());
                self.item_to_be_added.name = song.title().to_string();
                self.item_to_be_added.id = song.id().to_string();
                self.item_to_be_added.parent_id = selected_album_id.to_string();
                self.item_to_be_added.media_type = MediaType::Song;
            }
            MediaType::Album => {
                let selected_album_index = self.home_recent_state.selected().unwrap();
                self.item_to_be_added.id = self.database.recent_albums().get(selected_album_index).unwrap().id().to_string();
                self.item_to_be_added.media_type = MediaType::Album;
            }
            MediaType::Playlist => {}
        }
        Ok(())
    }

    pub fn toggle_playing_status(&mut self) -> AppResult<()> {
        self.player.toggle_play_pause();
        Ok(())
    }

    pub fn toggle_random_playback(&mut self) -> AppResult<()> {
        if self.queue.len() > 1 {
            if self.random_playback {
                self.index_in_queue = *self.queue_order.get(self.index_in_queue).unwrap();
                self.queue_order.clear();
                self.queue_order = (0..self.queue.len()).collect();
            }
            else {
                self.shuffle_queue_order_starting_at_current_index();
                self.index_in_queue = 0;
            }
        }
        self.random_playback = !self.random_playback;
        Ok(())
    }

    pub fn player_seek_forward(&mut self) -> AppResult<()> {
        if self.get_playback_time() + 10 > self.now_playing.duration.as_str().parse::<usize>().unwrap() {
            self.play_next()?;
        }
        else {
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
        }
        else { 
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
                IpcEvent::Eof(reason) => {
                    if reason == "eof" && self.queue_has_next() {
                        self.go_next_queue();
                        self.play_current(true);
                    }
                }
                IpcEvent::Seek => {
                    let playback_time = self.player.get_playback_time();
                    debug!("Got {} as playback time\n", playback_time);
                    if playback_time != -1.0 {
                        self.ticks_during_playing_state = (playback_time * 4.0).floor() as usize;
                    }
                }
                IpcEvent::Idle => {
                    if self.player.player_status == PlayerStatus::Playing && !self.queue_has_next() {
                        self.player.player_status = PlayerStatus::Stopped;
                    }
                }
                IpcEvent::Error(_) => {}
                IpcEvent::Unrecognized(_) => {}
            }
        }
    }

    pub fn queue_has_next(&self) -> bool {
        if self.queue.len() <= 1 { false } else {
            self.index_in_queue < self.queue_order.len() - 1
        }
    }

    fn queue_has_previous(&self) -> bool {
        if self.queue.len() <= 1 { false } else {
            self.index_in_queue > 0
        }
    }
    
    pub fn get_playback_time(&self) -> usize{
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
        self.change_current_playing_to(self.queue.get(self.queue_list_state.selected().unwrap()).unwrap().clone().as_str());
        self.index_in_queue = self.queue_list_state.selected().unwrap();
        self.play_current(false);
        Ok(())
    }

    pub fn clear_queue(&mut self) -> AppResult<()> {
        self.queue.clear();
        self.queue_order.clear();
        self.now_playing.id.clear();
        self.index_in_queue = 0;
        self.event_sender.as_ref().unwrap().send(Dbus(Clear)).unwrap();
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
            }
        }
        false
    }

    pub fn try_pause_current(&mut self) -> bool {
        if !self.now_playing.id.is_empty()  && self.player.player_status == PlayerStatus::Playing{
            self.toggle_playing_status().unwrap();
            return true
        }
        false
    }
    
    pub fn stop_playback(&mut self) {
        self.player.stop();
        self.player.player_status = PlayerStatus::Stopped;
    }
    

    fn play_current(&mut self, check_next_song: bool) {
        if check_next_song && self.next_is_in_player_queue {
            self.next_is_in_player_queue = false;
        }
        else {
            self.player.play_song(self.server.get_song_url(self.now_playing.id.clone()).as_str());
            self.next_is_in_player_queue = false;
        }
        self.event_sender.as_ref().unwrap().send(Dbus(Playing)).unwrap();
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
    
    pub fn get_metada_for_current_song(&mut self) -> HashMap<String,String> {
        let mut metadata = HashMap::new();
        let song = self.database.get_song(self.now_playing.id.as_str());
        metadata.insert("title".to_string(),song.title().to_string());
        metadata.insert("album".to_string(),song.album().to_string());
        metadata.insert("artist".to_string(),song.artist().to_string());
        metadata.insert("id".to_string(),song.id().to_string());
        metadata.insert("length".to_string(),song.duration().to_string());
        metadata.insert("cover".to_string(),self.server.get_song_art_url(song.id().to_string()));
        
        metadata
    }
}

async fn populate_album_in_db(server: &mut Server, music_database: &mut MusicDatabase, id: &str) -> AppResult<()> {
    let parsed_media = server.get_album(id).await.unwrap();
    music_database.insert_album(String::from(id), parsed_media.0);
    for song in parsed_media.1 {
        music_database.insert_song(song.id().to_string(), song);
    }
    Ok(())
}
