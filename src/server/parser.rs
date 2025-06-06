use chrono::{DateTime};
use crate::app::AppResult;
use crate::model::album::Album;
use crate::model::connection_status::ConnectionStatus;
use crate::model::song::Song;
use encoding::all::ISO_8859_1;
use encoding::{EncoderTrap, Encoding};
use log::{debug};
use crate::model::artist::Artist;
use crate::model::playlist::Playlist;

pub struct Parser {}

impl Parser {
    const NAMESPACE: &'static str = "http://subsonic.org/restapi";

    pub fn parse_connection_status(response: String) -> AppResult<ConnectionStatus> {
        let root: minidom::Element = response.parse()?;
        let mut connection_status: ConnectionStatus = ConnectionStatus::default();

        for attribute in root.attrs() {
            if attribute.0 == "status" {
                connection_status.set_status(attribute.1.to_string());
            } else if attribute.0 == "serverVersion" {
                connection_status.set_server_version(attribute.1.to_string());
            }
        }

        for children in root.children() {
            for attribute in children.attrs() {
                if attribute.0 == "code" {
                    connection_status.set_error_code(attribute.1.to_string());
                } else if attribute.0 == "message" {
                    connection_status.set_error_message(attribute.1.to_string());
                }
            }
        }

        Ok(connection_status)
    }

    pub fn parse_genres_list(response: String) -> AppResult<Vec<String>> {
        let mut genres_list: Vec<String> = Vec::new();
        let root: minidom::Element = response.parse()?;

        let list = match root.get_child("genres", Self::NAMESPACE) {
            None => { return Err("get child by 'genres' returned none".into())}
            Some(value) => value
        };
        for genre in list.children() {
            genres_list.push(parse_attribute(genre.text().as_str()));
        }
        Ok(genres_list)
    }

    pub fn parse_album_list_simple(response: String) -> AppResult<Vec<String>> {
        let root: minidom::Element = response.parse()?;
        let mut album_list = Vec::new();

        let list = match root.get_child("albumList", Self::NAMESPACE) {
            None => { return Err("get child by 'albumList' returned none".into())}
            Some(value) => value
        };
        for album in list.children() {
            let mut album_id = String::new();
            for attribute in album.attrs() {
                match attribute.0 {
                    "id" => album_id = attribute.1.to_string(),
                    &_ => {}
                }
            }
            album_list.push(album_id);
        }
        Ok(album_list)
    }

    pub fn parse_album(response: String) -> AppResult<(Album, Vec<Song>, Artist)> {
        let root: minidom::Element = response.parse()?;
        let mut song_list = Vec::new();
        let mut song_ids_list = Vec::new();

        let album = match root.get_child("album", Self::NAMESPACE) {
            None => { return Err("get child by 'album' returned none".into()) },
            Some(value) => value
        };
        let mut new_album = Album::default();
        let mut new_artist = Artist::default();
        new_artist.set_number_of_albums(1);
        let mut album_genres = Vec::new();

        for attribute in album.attrs() {
            match attribute.0 {
                "id" => new_album.set_id(attribute.1.to_string()),
                "name" => {
                    new_album.set_name(parse_attribute(attribute.1));
                }
                "artist" => {
                    let name = parse_attribute(attribute.1);
                    new_album.set_artist(name.clone());
                    new_artist.set_name(name);
                }
                "genre" => album_genres.push(parse_attribute(attribute.1)),
                "artistId" => new_artist.set_id(attribute.1.to_string()),
                "coverArt" => new_album.set_cover_art(attribute.1.to_string()),
                "duration" => new_album.set_duration(attribute.1.to_string()),
                "playCount" => new_album.set_play_count(attribute.1.to_string()),
                "songCount" => new_album.set_song_count(attribute.1.to_string()),
                "year" => new_album.set_year(attribute.1.to_string()),
                &_ => {}
            }
        }
        for child in album.children() {
            if child.name() == "genres" {
                for attribute in child.attrs() {
                    if attribute.0 == "name" {
                        let new_genre = parse_attribute(attribute.1);
                        if !album_genres.contains(&new_genre) {album_genres.push(new_genre);}
                    }
                }
            } else if child.name() == "song" {
                let mut new_song = Song::default();
                let mut song_genres = Vec::new();
                for attribute in child.attrs() {
                    match attribute.0 {
                        "id" => new_song.set_id(attribute.1.to_string()),
                        "title" => {
                            new_song.set_title(parse_attribute(attribute.1));
                        }
                        "album" => {
                            new_song.set_album(parse_attribute(attribute.1));
                        }
                        "albumId" => {
                            new_song.set_album_id(attribute.1.to_string());
                        }
                        "artist" => {
                            new_song.set_artist(parse_attribute(attribute.1));
                        }
                        "artistId" => new_song.set_artist_id(attribute.1.to_string()),
                        "coverArt" => new_song.set_cover_art(attribute.1.to_string()),
                        "track" => new_song.set_track(attribute.1.to_string()),
                        "duration" => new_song.set_duration(attribute.1.to_string()),
                        "playCount" => new_song.set_play_count(attribute.1.to_string()),
                        "bitRate" => new_song.set_bit_rate(attribute.1.to_string()),
                        &_ => {}
                    }
                }
                for child in child.children() {
                    if child.name() == "replayGain" {
                        for attribute in child.attrs() {
                            match attribute.0 {
                                "albumGain" => new_song.set_album_gain(attribute.1.to_string()),
                                "albumPeak" => new_song.set_album_peak(attribute.1.to_string()),
                                "trackGain" => new_song.set_track_gain(attribute.1.to_string()),
                                "trackPeak" => new_song.set_track_peak(attribute.1.to_string()),
                                &_ => {}
                            }
                        }
                    } else if child.name() == "genres" {
                        for attribute in child.attrs() {
                            match attribute.0 {
                                "name" => {
                                    song_genres.push(parse_attribute(attribute.1));
                                }
                                &_ => {}
                            }
                        }
                    }
                }
                if new_song.play_count().trim().is_empty() {
                    new_song.set_play_count("0".to_string());
                }
                new_song.set_genres(song_genres);
                song_ids_list.push(new_song.id().to_string());
                song_list.push(new_song);
            }
        }
        if new_album.play_count().trim().is_empty() {
            new_album.set_play_count("0".to_string());
        }
        new_album.set_songs(song_ids_list);
        new_album.set_genres(album_genres.clone());
        new_artist.set_genres(album_genres);

        Ok((new_album, song_list, new_artist))
    }

    pub fn parse_playlist_list(response: String) -> AppResult<Vec<Playlist>> {
        let mut playlist_list: Vec<Playlist> = vec![]; 
        let root: minidom::Element = response.parse()?;

        let playlists = match root.get_child("playlists", Self::NAMESPACE) {
            None => { return Err("get child by 'playlists' returned none".into())},
            Some(value) => value
        };
        
        for playlist in playlists.children() {
            let mut new_playlist: Playlist = Playlist::default();
            for attribute in playlist.attrs() {
                match attribute.0 {
                    "id" => new_playlist.set_id(attribute.1.to_string()),
                    "name" => new_playlist.set_name(attribute.1.to_string()),
                    "songCount" => new_playlist.set_song_count(attribute.1.to_string()),
                    "duration" => new_playlist.set_duration(attribute.1.to_string()),
                    "created" => new_playlist.set_created_on(parse_date(attribute.1.to_string().as_str())),
                    "changed" => new_playlist.set_modified_on(parse_date(attribute.1.to_string().as_str())),
                    _ => {}
                }
            }
            playlist_list.push(new_playlist);
        }
        
        Ok(playlist_list)
    }

    pub fn parse_playlist(response: String) -> AppResult<Vec<String>> {
        let mut playlists_songs: Vec<String> = vec![];
        let root: minidom::Element = response.parse()?;

        let list = match root.get_child("playlist", Self::NAMESPACE) {
            None => { return Err("get child by 'playlist' returned none".into())},
            Some(value) => value
        };
        for album in list.children() {
            let mut song_id = String::new();
            for attribute in album.attrs() {
                match attribute.0 {
                    "id" => song_id = attribute.1.to_string(),
                    &_ => {}
                }
            }
            playlists_songs.push(song_id);
        }
        Ok(playlists_songs)
    }

    pub fn parse_playlist_id(response: String) -> AppResult<String> {
        let root: minidom::Element = response.parse()?;

        let playlist = match root.get_child("playlist", Self::NAMESPACE) {
            None => { return Err("get child by 'playlist' returned none".into())},
            Some(value) => value
        };
        for attribute in playlist.attrs() {
            match attribute.0 {
                "id" => return Ok(attribute.1.to_string()),
                _ => {}
            }
        }
        Err(Box::from("Could not find playlist id in server response"))
    }
}
fn parse_date(string_date: &str) -> String {
    let dt = DateTime::parse_from_rfc3339(string_date);
    match dt {
        Ok(dt_ok) => dt_ok.format("%m/%d/%y - %H:%M").to_string(),
        Err(e) => {
            debug!("Could not parse date: {:?}", e);
            "".to_string()
        }
    }
}

fn parse_attribute(attribute: &str) -> String {
    let chars = ISO_8859_1.encode(attribute, EncoderTrap::Replace).unwrap();
    match String::from_utf8(chars) {
        Ok(parsed_string) => parsed_string,
        Err(e) => {
            debug!("Error while parsing: {}", attribute);
            debug!("Reason: {}", e);
            attribute.to_string()
        }
    }
}
