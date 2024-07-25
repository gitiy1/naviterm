use std::error;

use config::Config;
use ratatui::widgets::ListState;

use crate::music_database::MusicDatabase;
use crate::player::mpv::Mpv;
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
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    pub current_screen: CurrentScreen,
    pub current_popup: Popup,
    pub previous_popup: Popup,
    pub server: Server,
    pub database: MusicDatabase,
    pub home_recent_state: ListState,
    pub popup_list_state: ListState,
    pub item_to_be_added: ItemToBeAdded,
    pub queue: Vec<String>,
    pub now_playing: String,
    pub player: Mpv,
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
            popup_list_state: ListState::default(),
            item_to_be_added: ItemToBeAdded::default(),
            queue: vec![],
            now_playing: String::new(),
            player: Mpv::default()
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

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
    
    pub async fn add_queue_immediately(&mut self) -> AppResult<()> {
        match self.item_to_be_added.media_type {
            MediaType::Song => {
                self.queue.clear();
                self.queue.push(self.item_to_be_added.id.clone());
                self.now_playing.clone_from(&self.item_to_be_added.id);
                self.player.play_song(self.server.get_song_url(self.now_playing.clone()).as_str());
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
                self.player.play_song(self.server.get_song_url(self.now_playing.clone()).as_str());
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
        Ok(())
    }
    
    pub fn player_seek_backwards(&mut self) -> AppResult<()> {
        self.player.seek_backwards();
        Ok(())
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
