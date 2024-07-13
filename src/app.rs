use std::error;
use config::Config;
use ratatui::widgets::ListState;
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

/// Enum with applications screens
#[derive(Debug, PartialEq)]
pub enum CurrentPopup {
    ConnectionTest,
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
    pub current_popup: CurrentPopup,
    pub server: Server,
    pub database: MusicDatabase,
    pub home_recent_state: ListState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            current_screen: CurrentScreen::Home,
            current_popup: CurrentPopup::None,
            server: Server::new(),
            database: MusicDatabase::new(),
            home_recent_state: ListState::default(),
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

    /// Set running to false to quit the application.
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

    pub fn select_next_list(&mut self) -> AppResult<()> {
        self.home_recent_state.select_next();
        Ok(())
    }
    
    pub fn select_previous_list(&mut self) -> AppResult<()> {
        self.home_recent_state.select_previous();
        Ok(())
    }
}
