use crate::app::AppResult;
use crate::model::album::Album;
use crate::model::connection_status::ConnectionStatus;

pub struct Parser {}

impl Parser {

    const NAMESPACE: &'static str = "http://subsonic.org/restapi";

    pub fn parse_connection_status (response: String) -> AppResult<ConnectionStatus> {
        let root: minidom::Element = response.parse().unwrap();
        let mut connection_status: ConnectionStatus = ConnectionStatus::default() ;

        for attribute in root.attrs() {
            if attribute.0 == "status"{
                connection_status.set_status(attribute.1.to_string());
            }
            else if attribute.0 == "serverVersion" {
                connection_status.set_server_version(attribute.1.to_string());
            }

        }

        for children in root.children() {
            for attribute in children.attrs() {
                if attribute.0 == "code"{
                    connection_status.set_error_code(attribute.1.to_string());
                }
                else if attribute.0 == "message" {
                    connection_status.set_error_message(attribute.1.to_string());
                }
            }
        }

        Ok(connection_status)

    }

    pub fn parse_album_list (response: String) -> AppResult<Vec<Album>> {
        let root: minidom::Element = response.parse().unwrap();
        let mut album_list = Vec::new();

        let list = root.get_child("albumList", Self::NAMESPACE).unwrap();
        for album in list.children() {
            let mut new_album = Album::default();
            for attribute in album.attrs() {
                match attribute.0 {
                    "id" => {new_album.set_id(attribute.1.to_string())}
                    "album" => {new_album.set_name(attribute.1.to_string())}
                    "artist" => {new_album.set_artist(attribute.1.to_string())}
                    "coverArt" => {new_album.set_cover_art(attribute.1.to_string())}
                    "duration" => {new_album.set_duration(attribute.1.to_string())}
                    "songCount" => {new_album.set_song_count(attribute.1.to_string())}
                    "genre" => {new_album.set_genre(attribute.1.to_string())}
                    &_ => {}
                }
            }
            album_list.push(new_album);
        }
        
        Ok(album_list)
    }

}