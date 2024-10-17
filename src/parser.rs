use crate::app::AppResult;
use crate::model::album::Album;
use crate::model::connection_status::ConnectionStatus;
use crate::model::song::Song;
use encoding::all::ISO_8859_1;
use encoding::{EncoderTrap, Encoding};

pub struct Parser {}

impl Parser {
    const NAMESPACE: &'static str = "http://subsonic.org/restapi";

    pub fn parse_connection_status(response: String) -> AppResult<ConnectionStatus> {
        let root: minidom::Element = response.parse().unwrap();
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
        let root: minidom::Element = response.parse().unwrap();

        let list = root.get_child("genres", Self::NAMESPACE).unwrap();
        for genre in list.children() {
            let chars = ISO_8859_1
                .encode(genre.text().as_str(), EncoderTrap::Ignore)
                .unwrap();
            genres_list.push(String::from_utf8(chars).unwrap());
        }
        Ok(genres_list)
    }

    pub fn parse_album_list(response: String) -> AppResult<Vec<Album>> {
        let root: minidom::Element = response.parse().unwrap();
        let mut album_list = Vec::new();

        let list = root.get_child("albumList", Self::NAMESPACE).unwrap();
        for album in list.children() {
            let mut new_album = Album::default();
            let mut album_genres = Vec::new();
            for attribute in album.attrs() {
                match attribute.0 {
                    "id" => new_album.set_id(attribute.1.to_string()),
                    "album" => {
                        let chars = ISO_8859_1.encode(attribute.1, EncoderTrap::Ignore).unwrap();
                        new_album.set_name(String::from_utf8(chars).unwrap());
                    }
                    "artist" => {
                        let chars = ISO_8859_1.encode(attribute.1, EncoderTrap::Ignore).unwrap();
                        new_album.set_artist(String::from_utf8(chars).unwrap())
                    }
                    "coverArt" => new_album.set_cover_art(attribute.1.to_string()),
                    "duration" => new_album.set_duration(attribute.1.to_string()),
                    "playCount" => new_album.set_play_count(attribute.1.to_string()),
                    "songCount" => new_album.set_song_count(attribute.1.to_string()),
                    "genre" => {
                        let chars = ISO_8859_1.encode(attribute.1, EncoderTrap::Ignore).unwrap();
                        album_genres.push(String::from_utf8(chars).unwrap())
                    }
                    &_ => {}
                }
            }
            new_album.set_genres(album_genres);
            album_list.push(new_album);
        }

        Ok(album_list)
    }

    pub fn parse_album_list_simple(response: String) -> AppResult<Vec<String>> {
        let root: minidom::Element = response.parse().unwrap();
        let mut album_list = Vec::new();

        let list = root.get_child("albumList", Self::NAMESPACE).unwrap();
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

    pub fn parse_album(response: String) -> (Album, Vec<Song>) {
        let root: minidom::Element = response.parse().unwrap();
        let mut song_list = Vec::new();
        let mut song_ids_list = Vec::new();

        let album = root.get_child("album", Self::NAMESPACE).unwrap();
        let mut new_album = Album::default();
        let mut album_genres = Vec::new();

        for attribute in album.attrs() {
            match attribute.0 {
                "id" => new_album.set_id(attribute.1.to_string()),
                "name" => {
                    let chars = ISO_8859_1.encode(attribute.1, EncoderTrap::Ignore).unwrap();
                    new_album.set_name(String::from_utf8(chars).unwrap());
                }
                "artist" => {
                    let chars = ISO_8859_1.encode(attribute.1, EncoderTrap::Ignore).unwrap();
                    new_album.set_artist(String::from_utf8(chars).unwrap())
                }
                "coverArt" => new_album.set_cover_art(attribute.1.to_string()),
                "duration" => new_album.set_duration(attribute.1.to_string()),
                "playCount" => new_album.set_play_count(attribute.1.to_string()),
                "songCount" => new_album.set_song_count(attribute.1.to_string()),
                &_ => {}
            }
        }
        for child in album.children() {
            if child.name() == "genres" {
                for attribute in child.attrs() {
                    match attribute.0 {
                        "name" => {
                            let chars =
                                ISO_8859_1.encode(attribute.1, EncoderTrap::Ignore).unwrap();
                            album_genres.push(String::from_utf8(chars).unwrap());
                        }
                        &_ => {}
                    }
                }
            } else if child.name() == "song" {
                let mut new_song = Song::default();
                let mut song_genres = Vec::new();
                for attribute in child.attrs() {
                    match attribute.0 {
                        "id" => new_song.set_id(attribute.1.to_string()),
                        "title" => {
                            let chars =
                                ISO_8859_1.encode(attribute.1, EncoderTrap::Ignore).unwrap();
                            new_song.set_title(String::from_utf8(chars).unwrap());
                        }
                        "album" => {
                            let chars =
                                ISO_8859_1.encode(attribute.1, EncoderTrap::Ignore).unwrap();
                            new_song.set_album(String::from_utf8(chars).unwrap());
                        }
                        "albumId" => {
                            new_song.set_album_id(attribute.1.to_string());
                        }
                        "artist" => {
                            let chars =
                                ISO_8859_1.encode(attribute.1, EncoderTrap::Ignore).unwrap();
                            new_song.set_artist(String::from_utf8(chars).unwrap())
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
                                    let chars = ISO_8859_1
                                        .encode(attribute.1, EncoderTrap::Ignore)
                                        .unwrap();
                                    song_genres.push(String::from_utf8(chars).unwrap());
                                }
                                &_ => {}
                            }
                        }
                    }
                }
                new_song.set_genres(song_genres);
                song_ids_list.push(new_song.id().to_string());
                song_list.push(new_song);
            }
        }
        new_album.set_songs(song_ids_list);
        new_album.set_genres(album_genres);

        (new_album, song_list)
    }

    pub fn parse_album_songs(response: String) -> Vec<Song> {
        let root: minidom::Element = response.parse().unwrap();
        let mut song_list = Vec::new();
        let mut song_ids_list = Vec::new();

        let album = root.get_child("album", Self::NAMESPACE).unwrap();

        for child in album.children() {
            if child.name() == "song" {
                let mut new_song = Song::default();
                let mut song_genres = Vec::new();
                for attribute in child.attrs() {
                    match attribute.0 {
                        "id" => new_song.set_id(attribute.1.to_string()),
                        "title" => {
                            let chars =
                                ISO_8859_1.encode(attribute.1, EncoderTrap::Ignore).unwrap();
                            new_song.set_title(String::from_utf8(chars).unwrap());
                        }
                        "album" => {
                            let chars =
                                ISO_8859_1.encode(attribute.1, EncoderTrap::Ignore).unwrap();
                            new_song.set_album(String::from_utf8(chars).unwrap());
                        }
                        "albumId" => {
                            new_song.set_album_id(attribute.1.to_string());
                        }
                        "artist" => {
                            let chars =
                                ISO_8859_1.encode(attribute.1, EncoderTrap::Ignore).unwrap();
                            new_song.set_artist(String::from_utf8(chars).unwrap())
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
                                    let chars = ISO_8859_1
                                        .encode(attribute.1, EncoderTrap::Ignore)
                                        .unwrap();
                                    song_genres.push(String::from_utf8(chars).unwrap());
                                }
                                &_ => {}
                            }
                        }
                    }
                }
                new_song.set_genres(song_genres);
                song_ids_list.push(new_song.id().to_string());
                song_list.push(new_song);
            }
        }

        song_list
    }
}
