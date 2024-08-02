use std::error;
use config::Config;
use log::debug;
use ratatui::widgets::ListState;

use crate::music_database::MusicDatabase;
use crate::player::mpv::{Mpv, PlayerStatus};
use crate::player::ipc::IpcEvent;
use crate::server::Server;

/// Enum with applications screens
#[derive(Debug)]
pub enum CurrentScreen {
    Home,
    Albums,
    Playlists,
    Artists,
    Queue
}

#[derive(Debug, Default)]
pub enum MediaType {
    #[default]
    Song,
    Album,
    Playlist
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
    pub database: MusicDatabase,
    pub home_recent_state: ListState,
    pub queue_list_state: ListState,
    pub popup_list_state: ListState,
    pub item_to_be_added: ItemToBeAdded,
    pub queue: Vec<String>,
    pub now_playing: String,
    pub player: Mpv,
    pub index_in_queue: usize,
    pub ticks_during_playing_state: usize,
}

#[derive(Default,Debug)]
pub struct ItemToBeAdded {
    pub name: String,
    pub id: String,
    pub parent_id: String,
    pub media_type: MediaType
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            current_screen: CurrentScreen::Home,
            current_popup: Popup::None,
            previous_popup: Popup::None,
            server: Server::new(),
            database: MusicDatabase::new(),
            home_recent_state: ListState::default(),
            queue_list_state: ListState::default(),
            popup_list_state: ListState::default(),
            item_to_be_added: ItemToBeAdded::default(),
            queue: vec![],
            now_playing: String::new(),
            player: Mpv::default(),
            index_in_queue: 0,
            ticks_during_playing_state: 0,
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
            populate_album_in_db(&mut self.server, &mut self.database,selected_album_id.as_str()).await?;
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
        match self.item_to_be_added.media_type {
            MediaType::Song => {
                self.queue.clear();
                self.queue.push(self.item_to_be_added.id.clone());
                self.now_playing.clone_from(&self.item_to_be_added.id);
                self.play_current();
            }
            MediaType::Album => {
                self.queue.clear();
                if !self.database.contains_album(self.item_to_be_added.id.as_str()) {
                    populate_album_in_db(&mut self.server, &mut self.database,self.item_to_be_added.id.as_str()).await?;
                }
                let album = self.database.get_album(self.item_to_be_added.id.as_str());
                for song in album.songs() {
                    self.queue.push(song.clone());
                }
                self.now_playing.clone_from(self.queue.first().unwrap());
                self.play_current();
            }
            MediaType::Playlist => {}
        }
        Ok(())
    }
    
    pub fn add_queue_next(&mut self) -> AppResult<()> {
        match self.item_to_be_added.media_type {
            MediaType::Song => {
                let index = self.queue.iter().position(|x| x == &self.now_playing).unwrap();
                self.queue.insert(index+1,self.item_to_be_added.id.clone())
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
                let selected_album_id= self.database.recent_albums().get(selected_album_index).unwrap().id();
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
    
    pub fn player_seek_forward(&mut self) -> AppResult<()> {
        self.player.seek_forward();
        self.ticks_during_playing_state += 40;
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
            self.play_current();
        }
        Ok(())
    }
    
    pub fn play_previous(&mut self) -> AppResult<()> {
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
                        self.play_current();
                    }
                }
                IpcEvent::Seek => {
                    let playback_time = self.player.get_playback_time();
                    debug!("Got {} as playback time\n", playback_time);
                    if playback_time != -1.0 {
                        self.ticks_during_playing_state = (playback_time*4.0).floor() as usize;
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
    
    fn queue_has_next(&self) -> bool {
        if self.queue.len() <= 1 {return false;}
        else {
            self.index_in_queue < self.queue.len()-1
        }
    }

    fn go_next_queue(&mut self) {
        self.index_in_queue += 1;
        self.now_playing.clone_from(self.queue.get(self.index_in_queue).unwrap());
    }
    
    pub fn play_queue_song(&mut self) -> AppResult<()> {
        self.now_playing.clone_from(self.queue.get(self.queue_list_state.selected().unwrap()).unwrap());
        self.index_in_queue = self.queue_list_state.selected().unwrap();
        self.play_current();
        Ok(())
    }

    pub fn clear_queue(&mut self) -> AppResult<()> {
        self.queue.clear();
        self.index_in_queue = 0;
        self.player.stop();
        self.now_playing = String::new();
        Ok(())
    }

    fn play_current(&mut self) {
        self.player.play_song(self.server.get_song_url(self.now_playing.clone()).as_str());
        self.ticks_during_playing_state = 0;
    }
}

async fn populate_album_in_db(server: &mut Server, music_database: &mut MusicDatabase, id: &str) -> AppResult<()> {
    let parsed_media = server.get_album(id).await.unwrap();
    music_database.insert_album(String::from(id), parsed_media.0);
    for song in parsed_media.1  {
        music_database.insert_song(song.id().to_string(),song);
    }
    Ok(())
}
