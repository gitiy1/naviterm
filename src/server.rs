use std::fmt::Display;
use std::vec;
use reqwest::header::{CONTENT_TYPE, ACCEPT};
use chrono;
use rand::distributions::{Alphanumeric, DistString};
use reqwest::Client;
use md5;

use crate::app::AppResult;
use crate::parser::Parser;
use crate::model::album::Album;
use crate::model::song::Song;

#[derive(Clone, Copy)]
pub enum SubsonicOperation {
    Ping,
    GetGenres,
    GetAlbumListRecent,
    GetAlbumListMostListened,
    GetAlbumListAlphabetical,
    GetAlbumListByGenre,
    GetAlbumListByGenreAndMostListened,
    GetAlbum,
    DownloadSong,
    GetCoverArt,
}

#[derive(Debug)]
enum SubsonicParameter {
    None,
    AlbumId(String),
    SongId(String),
    Genre(String),
    Size(usize),
    Offset(usize)
}

impl Display for SubsonicParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            SubsonicParameter::AlbumId(val) => val.to_string(),
            SubsonicParameter::SongId(val) => val.to_string(),
            SubsonicParameter::Genre(val) => val.to_string(),
            SubsonicParameter::None => { "None".to_string() },
            SubsonicParameter::Size(val) => {val.to_string()}
            SubsonicParameter::Offset(val) => {val.to_string()}
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

        let url = self.build_url(SubsonicOperation::Ping, vec![SubsonicParameter::None]);
        let response_text = self.make_request_text(url).await.unwrap();

        let connection_status = Parser::parse_connection_status(response_text).unwrap();
        self.connection_status = connection_status.status().to_string();
        self.server_version = connection_status.server_version().to_string();
        self.connection_code = connection_status.error_code().to_string();
        self.connection_message = connection_status.error_message().to_string();
        self.last_connection_timestamp = chrono::offset::Local::now().to_string();

        Ok(())
    }

    pub async fn get_genres(&mut self) -> AppResult<Vec<String>> {

        let url = self.build_url(SubsonicOperation::GetGenres, vec![SubsonicParameter::None]);
        let response_text = self.make_request_text(url).await.unwrap();

        let genres_list = Parser::parse_genres_list(response_text).unwrap();

        Ok(genres_list)
    }

    pub async fn get_recent_albums(&mut self) -> AppResult<Vec<String>> {
        
        let url = self.build_url(SubsonicOperation::GetAlbumListRecent, vec![SubsonicParameter::None]);
        let response_text = self.make_request_text(url).await.unwrap();
        
        let album_list = Parser::parse_album_list_simple(response_text).unwrap();
        
        Ok(album_list)
    }

    pub async fn get_most_listened_albums(&mut self) -> AppResult<Vec<String>> {

        let parameters = vec![SubsonicParameter::Size(20),SubsonicParameter::Offset(0)];
        let url = self.build_url(SubsonicOperation::GetAlbumListMostListened, parameters);
        let response_text = self.make_request_text(url).await.unwrap();

        let album_list = Parser::parse_album_list_simple(response_text).unwrap();

        Ok(album_list)
    }

    pub async fn get_most_listened_albums_ids(&mut self, offset: usize) -> AppResult<Vec<String>> {

        let parameters = vec![SubsonicParameter::Size(10),SubsonicParameter::Offset(offset)];
        let url = self.build_url(SubsonicOperation::GetAlbumListMostListened, parameters);
        let response_text = self.make_request_text(url).await.unwrap();

        let album_list = Parser::parse_album_list_simple(response_text).unwrap();

        Ok(album_list)
    }

    pub async fn get_album_list_alphabetical(&mut self, offset: usize) -> AppResult<Vec<String>> {

        let parameters = vec![SubsonicParameter::Size(10),SubsonicParameter::Offset(offset)];
        let url = self.build_url(SubsonicOperation::GetAlbumListAlphabetical, parameters);
        let response_text = self.make_request_text(url).await.unwrap();

        let album_list = Parser::parse_album_list_simple(response_text).unwrap();

        Ok(album_list)
    }

    pub async fn get_album_list_by_genre(&mut self, offset: usize, genre: String, sort_by_most_listened: bool) -> AppResult<Vec<String>> {

        let parameters = vec![SubsonicParameter::Size(10),SubsonicParameter::Offset(offset), SubsonicParameter::Genre(genre)];
        let url = if sort_by_most_listened {
            self.build_url(SubsonicOperation::GetAlbumListByGenreAndMostListened, parameters)
        } else {
            self.build_url(SubsonicOperation::GetAlbumListByGenre, parameters)
        };
        let response_text = self.make_request_text(url).await.unwrap();

        let album_list = Parser::parse_album_list_simple(response_text).unwrap();

        Ok(album_list)
    }

    pub async fn get_album_list_complete(&mut self, operation: SubsonicOperation) -> AppResult<Vec<String>> {

        let mut stop = false;
        let mut offset = 0;

        let mut album_list: Vec<String> = vec![];
        while !stop {
            let parameters = vec![SubsonicParameter::Size(500),SubsonicParameter::Offset(offset)];
            let url = self.build_url(operation, parameters);
            let response_text = self.make_request_text(url).await.unwrap();
            let mut partial_album_list: Vec<String> = Parser::parse_album_list_simple(response_text).unwrap();
            stop = partial_album_list.is_empty();
            if !stop {
                album_list.append(&mut partial_album_list);
                offset += 500;
            }
        }

        Ok(album_list)
    }

    pub async fn get_complete_album(&mut self, album_id: &str) -> AppResult<(Album, Vec<Song>)> {

        let url = self.build_url(SubsonicOperation::GetAlbum, vec![SubsonicParameter::AlbumId(album_id.to_string())]);
        let response_text = self.make_request_text(url).await.unwrap();
        
        let parsed_media= Parser::parse_album(response_text);

        Ok(parsed_media)
    }

    pub async fn get_album_songs(&mut self, album_id: &str) -> AppResult<Vec<Song>> {

        let parameters = vec![SubsonicParameter::AlbumId(String::from(album_id))];
        let url = self.build_url(SubsonicOperation::GetAlbum, parameters);
        let response_text = self.make_request_text(url).await.unwrap();

        let parsed_media= Parser::parse_album_songs(response_text);

        Ok(parsed_media)
    }

    pub fn get_song_url(&mut self, id: String) -> String {
        self.build_url(SubsonicOperation::DownloadSong, vec![SubsonicParameter::SongId(id)])
    }

    pub fn get_song_art_url(&mut self, id: String) -> String {
        self.build_url(SubsonicOperation::GetCoverArt, vec![SubsonicParameter::SongId(id)])
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

    fn build_url(&mut self, subsonic_operation: SubsonicOperation, parameters: Vec<SubsonicParameter>) -> String {
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
            SubsonicOperation::GetAlbumListMostListened => {
                format!("{}/navidrome/rest/getAlbumList.view?type=frequent&\
                    size={}&offset={}&u={}&t={}&s={}&v=0.1&c=naviterm",
                        self.server_address, parameters[0], parameters[1], self.user, self.token, self.salt)
            }
            ,
            SubsonicOperation::GetAlbumListAlphabetical => {
                format!("{}/navidrome/rest/getAlbumList.view?type=alphabeticalByName&\
                    size={}&offset={}&u={}&t={}&s={}&v=0.1&c=naviterm",
                        self.server_address, parameters[0], parameters[1], self.user, self.token, self.salt)
            }
            ,
            SubsonicOperation::GetAlbumListByGenre => {
                format!("{}/navidrome/rest/getAlbumList.view?type=byGenre&\
                    size={}&offset={}&genre={}&u={}&t={}&s={}&v=0.1&c=naviterm",
                        self.server_address, parameters[0], parameters[1], parameters[2], self.user, self.token, self.salt)
            }
            ,
            SubsonicOperation::GetAlbumListByGenreAndMostListened => {
                format!("{}/navidrome/rest/getAlbumList.view?type=frequent&type=byGenre&\
                    size={}&offset={}&genre={}&u={}&t={}&s={}&v=0.1&c=naviterm",
                        self.server_address, parameters[0], parameters[1], parameters[2], self.user, self.token, self.salt)
            }
            ,
            SubsonicOperation::GetAlbum => {
                format!("{}/navidrome/rest/getAlbum.view?id={}&\
                    u={}&t={}&s={}&v=0.1&c=naviterm",
                        self.server_address, parameters[0], self.user, self.token, self.salt)
            }
            SubsonicOperation::DownloadSong => {
                format!("{}/navidrome/rest/download?id={}&\
                    u={}&t={}&s={}&v=0.1&c=naviterm",
                        self.server_address, parameters[0], self.user, self.token, self.salt)
            }
            SubsonicOperation::GetCoverArt => {
                format!("{}/navidrome/rest/getCoverArt.view?id={}&\
                    u={}&t={}&s={}&v=0.1&c=naviterm&size=300",
                        self.server_address, parameters[0], self.user, self.token, self.salt)
            }
            SubsonicOperation::GetGenres => {
                format!("{}/navidrome/rest/getGenres.view?\
                u={}&t={}&s={}&v=0.1&c=naviterm",
                        self.server_address, self.user, self.token, self.salt)
            }
        };

        url
    }

    pub fn set_password(&mut self, password: String) {
        self.password = password;
    }
}


