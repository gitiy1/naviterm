use std::fmt::Display;
use reqwest::header::{CONTENT_TYPE, ACCEPT};
use chrono;
use rand::distributions::{Alphanumeric, DistString};
use reqwest::Client;
use md5;

use crate::app::AppResult;
use crate::parser::Parser;
use crate::model::album::Album;
use crate::model::song::Song;

enum SubsonicOperation {
    Ping,
    GetAlbumListRecent,
    GetAlbum,
    DownloadSong
}

#[derive(Debug)]
enum SubsonicParameter {
    None,
    AlbumId(String),
    SongId(String)
}

impl Display for SubsonicParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SubsonicParameter::AlbumId(val) => val.to_string(),
            SubsonicParameter::SongId(val) => val.to_string(),
            SubsonicParameter::None => { "None".to_string() },
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug)]
pub struct Server{
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

}

impl Default for Server{
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
        }
    }
}

impl Server{
    
    /// Constructs a new instance of [`Server`].
    pub fn new() -> Self {
        Self::default()
    }

    pub fn renew_credentials(&mut self) -> AppResult<()> {
        let salt = Alphanumeric.sample_string(&mut rand::thread_rng(), 10).to_lowercase();
        let mut concatenation: String = String::from(&self.password);
        concatenation.push_str(&salt);
        let token = md5::compute(concatenation.as_bytes());

        self.salt = salt;
        self.token = format!("{:x}", token);

        Ok(())
    }

    pub async fn test_connection(&mut self) -> AppResult<()> {

        let url = self.build_url(SubsonicOperation::Ping, SubsonicParameter::None);
        let response_text = self.make_request_text(url).await.unwrap();

        let connection_status = Parser::parse_connection_status(response_text).unwrap();
        self.connection_status = connection_status.status().to_string();
        self.server_version = connection_status.server_version().to_string();
        self.connection_code = connection_status.error_code().to_string();
        self.connection_message = connection_status.error_message().to_string();
        self.last_connection_timestamp = chrono::offset::Local::now().to_string();

        Ok(())
    }
    
    pub async fn get_recent_albums(&mut self) -> AppResult<Vec<Album>> {
        
        let url = self.build_url(SubsonicOperation::GetAlbumListRecent, SubsonicParameter::None);
        let response_text = self.make_request_text(url).await.unwrap();
        
        let album_list = Parser::parse_album_list(response_text).unwrap();
        
        Ok(album_list)
    }

    pub async fn get_album(&mut self, album_id: &str) -> AppResult<(Album, Vec<Song>)> {

        let url = self.build_url(SubsonicOperation::GetAlbum, SubsonicParameter::AlbumId(String::from(album_id)));
        let response_text = self.make_request_text(url).await.unwrap();

        let parsed_media= Parser::parse_album(response_text);

        Ok(parsed_media)
    }
    
    pub fn get_song_url(&mut self, id: String) -> String {
        self.build_url(SubsonicOperation::DownloadSong, SubsonicParameter::SongId(id))
    }
    
    async fn make_request_text(&mut self, url: String) -> AppResult<String> {

        let mut response_text = "".to_string();

        let response = self.client.get(url)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .send().await;

        match response {
            Ok(success_response) => match success_response.status() {
                reqwest::StatusCode::OK => {
                     response_text = success_response.text().await.unwrap();
                },
                reqwest::StatusCode::UNAUTHORIZED => {
                    println!("Need to grab a new token");
                    //TODO
                },
                _ => {
                    panic!("Uh oh! Something unexpected happened.");
                    //TODO
                },
            },
            Err(error) => panic!("Error while doing request: {:?}", error)
            //TODO
        };
        Ok(response_text)
    }

    fn build_url(&mut self, subsonic_operation: SubsonicOperation, subsonic_parameter: SubsonicParameter) -> String {
        let url: String = match subsonic_operation {
            SubsonicOperation::Ping => 
                format!("{}/navidrome/rest/ping.view?\
                    u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address, self.user, self.token, self.salt)
            ,
            SubsonicOperation::GetAlbumListRecent => {
                format!("{}/navidrome/rest/getAlbumList.view?type=recent&\
                    u={}&t={}&s={}&v=0.1&c=naviterm",
                    self.server_address, self.user, self.token, self.salt)
            }
            ,
            SubsonicOperation::GetAlbum => {
                format!("{}/navidrome/rest/getAlbum.view?id={}&\
                    u={}&t={}&s={}&v=0.1&c=naviterm",
                        self.server_address, subsonic_parameter, self.user, self.token, self.salt)
            }
            SubsonicOperation::DownloadSong => {
                format!("{}/navidrome/rest/download?id={}&\
                    u={}&t={}&s={}&v=0.1&c=naviterm",
                        self.server_address, subsonic_parameter, self.user, self.token, self.salt)
            }
        };

        url
    }

    pub fn set_password(&mut self, password: String) {
        self.password = password;
    }
}


