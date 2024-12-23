use crate::app::AppResult;
use crate::constants::{ALBUM_LIST_CHUNK_SIZE, MAX_SIMULTANEOUS_OPERATIONS};
use crate::model::song::Song;
use crate::server::async_operation::{AsyncOperation, Operation};
use crate::server::parser::Parser;
use chrono;
use log::{debug, error, warn};
use md5;
use rand::distributions::{Alphanumeric, DistString};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::Client;
use std::fmt::Display;
use std::time::Duration;
use std::vec;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::time::sleep;

#[derive(Clone, Copy)]
pub enum SubsonicOperation {
    Ping,
    GetGenres,
    GetAlbumListRecent,
    GetAlbumListMostListened,
    GetAlbumListAlphabetical,
    GetAlbumListByGenre,
    GetAlbumListByGenreAndMostListened,
    GetPlaylistList,
    GetPlaylist,
    CreatePlaylist,
    UpdatePlaylist,
    DeletePlaylist,
    GetAlbum,
    DownloadSong,
    GetCoverArt,
    GetAlbumListRecentlyAdded,
    Scrobble,
}

#[derive(Debug)]
enum SubsonicParameter {
    None,
    AlbumId(String),
    SongId(String),
    PlaylistId(String),
    PlaylistName(String),
    PlaylistSongs(Vec<String>),
    Size(usize),
    Offset(usize),
}

impl Display for SubsonicParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SubsonicParameter::AlbumId(val) => val.to_string(),
            SubsonicParameter::SongId(val) => val.to_string(),
            SubsonicParameter::None => "None".to_string(),
            SubsonicParameter::Size(val) => val.to_string(),
            SubsonicParameter::Offset(val) => val.to_string(),
            SubsonicParameter::PlaylistId(val) => val.to_string(),
            SubsonicParameter::PlaylistName(val) => val.to_string(),
            SubsonicParameter::PlaylistSongs(song_vector) => song_vector.join("&songId="),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug)]
pub struct Server {
    pub server_address: String,
    pub server_version: String,
    /// server token
    pub token: String,
    /// salt
    pub salt: String,
    pub connection_status: String,
    pub connection_message: String,
    pub connection_code: String,
    pub last_connection_timestamp: String,
    /// user
    pub user: String,
    /// password
    password: String,
    /// http client
    client: Client,
    pub operations: Vec<AsyncOperation>,
    current_number_of_requests: usize,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            token: "".to_string(),
            salt: "".to_string(),
            connection_status: "".to_string(),
            connection_message: "".to_string(),
            connection_code: "".to_string(),
            last_connection_timestamp: "".to_string(),
            server_address: "".to_string(),
            server_version: "".to_string(),
            user: "".to_string(),
            password: "".to_string(),
            client: Client::new(),
            operations: vec![],
            current_number_of_requests: 0,
        }
    }
}

impl Server {
    /// Constructs a new instance of [`Server`].
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process_async_operations(&mut self) {
        for operation in self.operations.iter_mut() {
            if !operation.finished() {
                debug!("Processing operation {:?}", operation);
                let result = process_atomic_operations(operation);
                debug!("Delta requests: {}", result);
                if result > 0 {
                    self.current_number_of_requests += result as usize;
                } else {
                    self.current_number_of_requests = self
                        .current_number_of_requests
                        .saturating_sub(result.unsigned_abs());
                }
                debug!(
                    "Total inflight requests: {}",
                    self.current_number_of_requests
                );
            }
            if self.current_number_of_requests > MAX_SIMULTANEOUS_OPERATIONS {
                debug!("Maximum number of requests reached");
                continue;
            }
        }
    }

    pub fn remove_completed_operations(&mut self) {
        self.operations.retain(|operation| !operation.processed());
    }

    pub fn renew_credentials(&mut self) -> AppResult<()> {
        let salt = Alphanumeric
            .sample_string(&mut rand::thread_rng(), 10)
            .to_lowercase();
        let mut concatenation: String = String::from(&self.password);
        concatenation.push_str(&salt);
        let token = md5::compute(concatenation.as_bytes());

        self.salt = salt;
        self.token = format!("{:x}", token);

        Ok(())
    }

    pub async fn test_connection(&mut self) -> AppResult<()> {
        let url = self.build_url(SubsonicOperation::Ping, vec![SubsonicParameter::None]);
        let response_text = self.make_request_text(url).await?;

        let connection_status = Parser::parse_connection_status(response_text)?;
        self.connection_status = connection_status.status().to_string();
        self.server_version = connection_status.server_version().to_string();
        self.connection_code = connection_status.error_code().to_string();
        self.connection_message = connection_status.error_message().to_string();
        self.last_connection_timestamp = chrono::offset::Local::now().to_string();

        Ok(())
    }

    pub fn get_genres_async(&mut self) {
        let url = self.build_url(SubsonicOperation::GetGenres, vec![SubsonicParameter::None]);

        let (tx, rx) = mpsc::unbounded_channel();
        let operation = AsyncOperation::new(Operation::GetGenreList, url.clone(), rx, tx);

        self.operations.push(operation);
    }

    pub fn get_playlists_async(&mut self, update: bool) {
        let url = self.build_url(
            SubsonicOperation::GetPlaylistList,
            vec![SubsonicParameter::None],
        );

        let (tx, rx) = mpsc::unbounded_channel();
        let operation =
            AsyncOperation::new(Operation::GetPlaylistList(update), url.clone(), rx, tx);

        self.operations.push(operation);
    }

    pub fn get_playlist_async(&mut self, playlist_id: &str) {
        let url = self.build_url(
            SubsonicOperation::GetPlaylist,
            vec![SubsonicParameter::PlaylistId(String::from(playlist_id))],
        );

        let (tx, rx) = mpsc::unbounded_channel();
        let operation = AsyncOperation::new(
            Operation::GetPlaylist(playlist_id.to_string()),
            url.clone(),
            rx,
            tx,
        );

        self.operations.push(operation);
    }

    pub fn create_playlist_async(&mut self, name: &str, songs: Vec<String>, temporary_playlist_id: &str) {
        let url = self.build_url(
            SubsonicOperation::CreatePlaylist,
            vec![
                SubsonicParameter::PlaylistName(name.to_string()),
                SubsonicParameter::PlaylistSongs(songs),
            ],
        );
        
        let (tx, rx) = mpsc::unbounded_channel();
        let operation = AsyncOperation::new(
            Operation::CreatePlaylist(temporary_playlist_id.to_string()),
            url.clone(),
            rx,
            tx,
        );

        self.operations.push(operation);
    }

    pub fn update_playlist_async(&mut self,  songs: Vec<String>, playlist_id: &str) {
        let url = self.build_url(
            SubsonicOperation::UpdatePlaylist,
            vec![
                SubsonicParameter::PlaylistId(playlist_id.to_string()),
                SubsonicParameter::PlaylistSongs(songs),
            ],
        );

        let (tx, rx) = mpsc::unbounded_channel();
        let operation = AsyncOperation::new(
            Operation::UpdatePlaylist(playlist_id.to_string()),
            url.clone(),
            rx,
            tx,
        );

        self.operations.push(operation);
    }

    pub fn delete_playlist_async(&mut self, playlist_id: &str) {
        let url = self.build_url(
            SubsonicOperation::DeletePlaylist,
            vec![SubsonicParameter::PlaylistId(playlist_id.to_string())],
        );
        let (tx, rx) = mpsc::unbounded_channel();
        let operation = AsyncOperation::new(Operation::DeletePlaylist(playlist_id.to_string()), url, rx, tx);
        
        self.operations.push(operation);
    }

    pub fn get_recent_albums_async(&mut self) {
        let url = self.build_url(
            SubsonicOperation::GetAlbumListRecent,
            vec![SubsonicParameter::Size(ALBUM_LIST_CHUNK_SIZE)],
        );

        let (tx, rx) = mpsc::unbounded_channel();
        let operation = AsyncOperation::new(Operation::GetAlbumListRecent(), url.clone(), rx, tx);

        self.operations.push(operation);
    }

    pub fn get_recently_added_albums_async(&mut self) {
        let url = self.build_url(
            SubsonicOperation::GetAlbumListRecentlyAdded,
            vec![SubsonicParameter::Size(ALBUM_LIST_CHUNK_SIZE)],
        );

        let (tx, rx) = mpsc::unbounded_channel();
        let operation =
            AsyncOperation::new(Operation::GetAlbumListRecentlyAdded(), url.clone(), rx, tx);

        self.operations.push(operation);
    }

    pub fn get_album_list_alphabetical_async(&mut self, update: bool, offset: usize) {
        let parameters = vec![
            SubsonicParameter::Size(ALBUM_LIST_CHUNK_SIZE),
            SubsonicParameter::Offset(offset),
        ];
        let url = self.build_url(SubsonicOperation::GetAlbumListAlphabetical, parameters);

        let (tx, rx) = mpsc::unbounded_channel();
        let operation = AsyncOperation::new(
            Operation::GetAlbumListAlphabetical(update, offset),
            url.clone(),
            rx,
            tx,
        );

        self.operations.push(operation);
    }

    pub fn get_most_listened_albums_async(&mut self, offset: usize) {
        let parameters = vec![
            SubsonicParameter::Size(ALBUM_LIST_CHUNK_SIZE),
            SubsonicParameter::Offset(offset),
        ];
        let url = self.build_url(SubsonicOperation::GetAlbumListMostListened, parameters);

        let (tx, rx) = mpsc::unbounded_channel();
        let operation = AsyncOperation::new(
            Operation::GetAlbumListMostListened(offset),
            url.clone(),
            rx,
            tx,
        );

        self.operations.push(operation);
    }

    pub fn get_album_async(&mut self, album_id: String) {
        let url = self.build_url(
            SubsonicOperation::GetAlbum,
            vec![SubsonicParameter::AlbumId(album_id.clone())],
        );

        let (tx, rx) = mpsc::unbounded_channel();
        let operation = AsyncOperation::new(Operation::GetAlbum(album_id), url.clone(), rx, tx);

        self.operations.push(operation);
    }

    pub fn scrobble_song_async(&mut self, id: String) {
        let url = self.build_url(
            SubsonicOperation::Scrobble,
            vec![SubsonicParameter::SongId(id.clone())],
        );

        let (tx, rx) = mpsc::unbounded_channel();
        let operation = AsyncOperation::new(Operation::Scrobble(id), url.clone(), rx, tx);

        self.operations.push(operation);
    }

    pub fn get_song_url(&mut self, id: String) -> String {
        self.build_url(
            SubsonicOperation::DownloadSong,
            vec![SubsonicParameter::SongId(id)],
        )
    }

    pub fn get_song_art_url(&mut self, id: String) -> String {
        self.build_url(
            SubsonicOperation::GetCoverArt,
            vec![SubsonicParameter::SongId(id)],
        )
    }

    async fn make_request_text(&mut self, url: String) -> AppResult<String> {
        let mut response_text = "".to_string();

        let response = self
            .client
            .get(url)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .send()
            .await;

        match response {
            Ok(success_response) => match success_response.status() {
                reqwest::StatusCode::OK => {
                    response_text = success_response.text().await.unwrap();
                    debug!("Response from server: {}", response_text)
                }
                reqwest::StatusCode::UNAUTHORIZED => {
                    debug!("Need to grab a new token");
                    //TODO
                }
                _ => {
                    panic!("Uh oh! Something unexpected happened.");
                    //TODO
                }
            },
            Err(error) => {
                error!("Error while doing request: {:?}", error)
            }
        };
        Ok(response_text)
    }

    fn build_url(
        &mut self,
        subsonic_operation: SubsonicOperation,
        parameters: Vec<SubsonicParameter>,
    ) -> String {
        let url: String = match subsonic_operation {
            SubsonicOperation::Ping => format!(
                "{}/navidrome/rest/ping.view?\
                    u={}&t={}&s={}&v=0.1&c=naviterm",
                self.server_address, self.user, self.token, self.salt
            ),
            SubsonicOperation::GetAlbumListRecent => {
                format!(
                    "{}/navidrome/rest/getAlbumList.view?type=recent&\
                    size={}&u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address, parameters[0], self.user, self.token, self.salt
                )
            }
            SubsonicOperation::GetAlbumListMostListened => {
                format!(
                    "{}/navidrome/rest/getAlbumList.view?type=frequent&\
                    size={}&offset={}&u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address,
                    parameters[0],
                    parameters[1],
                    self.user,
                    self.token,
                    self.salt
                )
            }
            SubsonicOperation::GetAlbumListAlphabetical => {
                format!(
                    "{}/navidrome/rest/getAlbumList.view?type=alphabeticalByName&\
                    size={}&offset={}&u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address,
                    parameters[0],
                    parameters[1],
                    self.user,
                    self.token,
                    self.salt
                )
            }
            SubsonicOperation::GetAlbumListByGenre => {
                format!(
                    "{}/navidrome/rest/getAlbumList.view?type=byGenre&\
                    size={}&offset={}&genre={}&u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address,
                    parameters[0],
                    parameters[1],
                    parameters[2],
                    self.user,
                    self.token,
                    self.salt
                )
            }
            SubsonicOperation::GetAlbumListByGenreAndMostListened => {
                format!(
                    "{}/navidrome/rest/getAlbumList.view?type=frequent&type=byGenre&\
                    size={}&offset={}&genre={}&u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address,
                    parameters[0],
                    parameters[1],
                    parameters[2],
                    self.user,
                    self.token,
                    self.salt
                )
            }
            SubsonicOperation::GetAlbum => {
                format!(
                    "{}/navidrome/rest/getAlbum.view?id={}&\
                    u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address, parameters[0], self.user, self.token, self.salt
                )
            }
            SubsonicOperation::DownloadSong => {
                format!(
                    "{}/navidrome/rest/download?id={}&\
                    u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address, parameters[0], self.user, self.token, self.salt
                )
            }
            SubsonicOperation::GetCoverArt => {
                format!(
                    "{}/navidrome/rest/getCoverArt.view?id={}&\
                    u={}&t={}&s={}&v=0.1&c=naviterm&size=300",
                    self.server_address, parameters[0], self.user, self.token, self.salt
                )
            }
            SubsonicOperation::GetGenres => {
                format!(
                    "{}/navidrome/rest/getGenres.view?\
                u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address, self.user, self.token, self.salt
                )
            }
            SubsonicOperation::GetPlaylistList => {
                format!(
                    "{}/navidrome/rest/getPlaylists.view?\
                u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address, self.user, self.token, self.salt
                )
            }
            SubsonicOperation::GetPlaylist => {
                format!(
                    "{}/navidrome/rest/getPlaylist.view?id={}&\
                u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address, parameters[0], self.user, self.token, self.salt
                )
            }
            SubsonicOperation::GetAlbumListRecentlyAdded => {
                format!(
                    "{}/navidrome/rest/getAlbumList.view?type=newest&\
                    u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address, self.user, self.token, self.salt
                )
            }
            SubsonicOperation::Scrobble => {
                format!(
                    "{}/navidrome/rest/scrobble?id={}&\
                    u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address, parameters[0], self.user, self.token, self.salt
                )
            }
            SubsonicOperation::CreatePlaylist => {
                format!(
                    "{}/navidrome/rest/createPlaylist.view?name={}&\
                    songId={}&u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address,
                    parameters[0],
                    parameters[1],
                    self.user,
                    self.token,
                    self.salt
                )
            }
            SubsonicOperation::DeletePlaylist => {
                format!(
                    "{}/navidrome/rest/deletePlaylist.view?id={}&\
                u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address, parameters[0], self.user, self.token, self.salt
                )
            }
            SubsonicOperation::UpdatePlaylist => {
                format!(
                    "{}/navidrome/rest/createPlaylist.view?playlistId={}&\
                    songId={}&u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address,
                    parameters[0],
                    parameters[1],
                    self.user,
                    self.token,
                    self.salt
                )
            }
        };

        url
    }

    pub fn set_password(&mut self, password: String) {
        self.password = password;
    }
}

fn process_atomic_operations(operation: &mut AsyncOperation) -> isize {
    let mut new_requests = 0;

    if !operation.started() {
        let url = operation.operation_url().to_string();
        let sender = operation.thread_tx_handle().clone();
        operation.set_started(true);
        debug!("Starting operation. Processing request to {}.", url);
        // Spawn a new tokio task
        tokio::spawn(async move {
            perform_request_async(url, sender).await;
        });
        new_requests += 1;
    } else if operation.started() && !operation.finished() {
        match operation.thread_rx_handle().try_recv() {
            Ok(message) => {
                debug!("Received message from thread_rx_handle");
                operation.set_result(message);
                operation.set_finished(true);
                operation.thread_rx_handle().close();
                new_requests -= 1;
            }
            Err(e) => {
                debug!("Operation not ready with error: {}", e);
            }
        }
    }
    new_requests
}

async fn perform_request_async(url: String, sender: UnboundedSender<String>) {
    let client = Client::new();
    let mut response = client
        .get(&url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .send()
        .await;
    
    let mut retries = 1;
    
    while response.is_err() && retries <= 3 {
        sleep(Duration::from_millis(100)).await;
        warn!("Error while doing request: {:?}", response.err().unwrap());
        warn!("Retrying request {}/3", retries);
        retries += 1;
        response = client
            .get(&url)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .send()
            .await;
    }

    let result = match response {
        Ok(success_response) => match success_response.status() {
            reqwest::StatusCode::OK => {
                let response_text = success_response.text().await;
                match response_text {
                    Ok(text) => {
                        debug!("Response from server: {}", text);
                        text
                    }
                    Err(_) => "error".into(),
                }
            }
            _ => "error".into(),
        },
        Err(err) => {
            panic!("Error while doing request: {:?}", err);
        }
    };

    // Send the result back to the channel
    let _ = sender.send(result);
}
