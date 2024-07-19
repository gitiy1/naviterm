use std::error;
use config::Config;
use ratatui::widgets::ListState;
use crate::model::song::Song;
use crate::music_database::MusicDatabase;
use crate::server::Server;

/// Enum with applications screens
#[derive(Debug)]
pub enum CurrentScreen {
    Home,
    Albums,
    Playlists,
    Artists,
}

pub enum MediaType {
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
    pub queue: Vec<Song>
}

#[derive(Default,Debug)]
pub struct ItemToBeAdded {
    pub name: String,
    pub id: String,
    pub parent_id: String
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
    
    pub async fn get_current_album_information(&mut self) -> AppResult<()> {
        let selected_album_index = self.home_recent_state.selected().unwrap();
        let selected_album_id= self.database.recent_albums().get(selected_album_index).unwrap().id();
        if !self.database.contains_album(selected_album_id) {
            let parsed_media = self.server.get_album(selected_album_id).await.unwrap();
            self.database.insert_album(String::from(selected_album_id), parsed_media.0);
            for song in parsed_media.1  {
                self.database.insert_song(song.id().to_string(),song);
            }
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
    
    pub fn add_queue_immediately(&mut self, media: MediaType) -> AppResult<()> {
        match media {
            MediaType::Song => {}
            MediaType::Album => {}
            MediaType::Playlist => {}
        }
        Ok(())
    }
    
    pub fn add_queue_next(&mut self) -> AppResult<()> {
        Ok(())
    }
    
    pub fn add_queue_later(&mut self) -> AppResult<()> {
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
            }
            MediaType::Album => {
                let selected_album_index = self.home_recent_state.selected().unwrap();
                self.item_to_be_added.id = self.database.recent_albums().get(selected_album_index).unwrap().id().to_string();
            }
            MediaType::Playlist => {}
        }
        Ok(())
    }
}
