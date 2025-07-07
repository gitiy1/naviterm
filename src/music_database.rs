use crate::constants::{DEFAULT_ALBUM, DEFAULT_SONG};
use crate::model::album::Album;
use crate::model::artist::Artist;
use crate::model::playlist::Playlist;
use crate::model::song::Song;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MusicDatabase {
    recent_albums: Vec<String>,
    most_listened_albums: Vec<String>,
    alphabetical_albums: Vec<String>,
    most_listened_tracks: Vec<String>,
    recently_added_albums: Vec<String>,
    filtered_albums: Vec<String>,
    genres: Vec<String>,
    favorite_genres: Vec<String>,
    albums: HashMap<String, Album>,
    songs: HashMap<String, Song>,
    playlists: HashMap<String, Playlist>,
    artists: HashMap<String, Artist>,
    alphabetical_artists: Vec<String>,
    alphabetical_playlists: Vec<String>,
    last_played_album_id: String,
    number_of_local_playlists: usize,
}

impl MusicDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn populate_defaults(&mut self) {
        let mut song = Song::default();
        song.set_id(String::from(DEFAULT_SONG));
        song.set_artist(String::from("Not found"));
        song.set_duration(String::from("0"));
        song.set_album("Not found".to_string());
        song.set_title(String::from("Not found"));
        song.set_play_count(String::from("0"));
        song.set_album_id(String::from(DEFAULT_ALBUM));
        self.songs.insert(DEFAULT_SONG.to_string(), song);

        let mut album = Album::default();
        album.set_name(String::from("Not found"));
        album.set_id(String::from(DEFAULT_ALBUM));
        album.set_artist(String::from("Not found"));
        album.set_song_count(String::from("1"));
        album.set_songs(vec![DEFAULT_SONG.to_string()]);
        album.set_duration(String::from("0"));
        album.set_play_count(String::from("0"));
        album.set_genres(vec![String::from("?")]);
        album.set_year("?".to_string());
        self.albums.insert(DEFAULT_ALBUM.to_string(), album);
    }

    pub fn recent_albums(&self) -> &Vec<String> {
        &self.recent_albums
    }

    pub fn recent_albums_mut(&mut self) -> &mut Vec<String> {
        &mut self.recent_albums
    }

    pub fn most_listened_albums(&self) -> &Vec<String> {
        &self.most_listened_albums
    }

    pub fn most_listened_albums_mut(&mut self) -> &mut Vec<String> {
        &mut self.most_listened_albums
    }

    pub fn alphabetical_list_albums(&self) -> &Vec<String> {
        &self.alphabetical_albums
    }

    pub fn alphabetical_list_albums_mut(&mut self) -> &mut Vec<String> {
        &mut self.alphabetical_albums
    }

    pub fn filtered_albums(&self) -> &Vec<String> {
        &self.filtered_albums
    }

    pub fn filtered_albums_mut(&mut self) -> &mut Vec<String> {
        &mut self.filtered_albums
    }

    pub fn expand_filtered_albums(&mut self, mut list: Vec<String>) {
        self.filtered_albums.append(&mut list);
    }

    pub fn set_recent_albums(&mut self, recent_albums: Vec<String>) {
        self.recent_albums = recent_albums;
    }

    pub fn set_filtered_albums(&mut self, filtered_albums: Vec<String>) {
        self.filtered_albums = filtered_albums;
    }

    pub fn set_most_listened_albums(&mut self, most_listened_albums: Vec<String>) {
        self.most_listened_albums = most_listened_albums;
    }

    pub fn set_alphabetical_albums(&mut self, alphabetical_list: Vec<String>) {
        self.alphabetical_albums = alphabetical_list;
    }

    pub fn insert_album(&mut self, id: String, album: Album) {
        self.albums.insert(id, album);
    }

    pub fn delete_album(&mut self, id: String) {
        self.albums.remove(&id);
    }

    pub fn delete_song(&mut self, id: String) {
        self.songs.remove(&id);
    }

    pub fn get_album(&self, id: &str) -> &Album {
        match self.albums.get(id) {
            Some(album) => album,
            None => self.albums.get(DEFAULT_ALBUM).unwrap(),
        }
    }

    pub fn set_album_songs(&mut self, id: &str, songs: Vec<String>) {
        self.albums.get_mut(id).unwrap().set_songs(songs);
    }

    pub fn contains_album(&self, id: &str) -> bool {
        self.albums.contains_key(id)
    }

    pub fn contains_complete_album(&self, id: &str) -> bool {
        !self.albums.get(id).unwrap().songs().is_empty()
    }

    pub fn insert_song(&mut self, id: String, song: Song) {
        self.songs.insert(id, song);
    }

    pub fn get_song(&self, id: &str) -> &Song {
        match self.songs.get(id) {
            None => self.songs.get(DEFAULT_SONG).unwrap(),
            Some(song) => song,
        }
    }

    pub fn get_song_mut(&mut self, id: &str) -> &mut Song {
        self.songs.get_mut(id).unwrap()
    }

    pub fn contains_song(&self, id: &str) -> bool {
        self.songs.contains_key(id)
    }

    pub fn genres(&self) -> &Vec<String> {
        &self.genres
    }

    pub fn set_genres(&mut self, genres: Vec<String>) {
        self.genres = genres;
    }

    pub fn insert_playlist(&mut self, id: String, playlist: Playlist) {
        self.playlists.insert(id, playlist);
    }

    pub fn remove_playlist(&mut self, id: &str) -> Playlist {
        self.playlists.remove(id).unwrap()
    }

    pub fn delete_playlist(&mut self, id: String) {
        self.playlists.remove(&id);
    }

    pub fn get_playlist(&self, id: &str) -> &Playlist {
        self.playlists.get(id).unwrap()
    }

    pub fn get_mut_playlist(&mut self, id: &str) -> &mut Playlist {
        self.playlists.get_mut(id).unwrap()
    }

    pub fn set_playlist_songs(&mut self, id: &str, songs: Vec<String>) {
        self.playlists.get_mut(id).unwrap().set_song_list(songs);
    }
    pub fn contains_playlist(&self, id: &str) -> bool {
        self.playlists.contains_key(id)
    }

    pub fn playlists(&self) -> &HashMap<String, Playlist> {
        &self.playlists
    }

    pub fn recently_added_albums(&self) -> &Vec<String> {
        &self.recently_added_albums
    }

    pub fn most_listened_tracks(&self) -> &Vec<String> {
        &self.most_listened_tracks
    }

    pub fn set_most_listened_tracks(&mut self, most_listened_tracks: Vec<String>) {
        self.most_listened_tracks = most_listened_tracks;
    }

    pub fn set_recently_added_albums(&mut self, recently_added_albums: Vec<String>) {
        self.recently_added_albums = recently_added_albums;
    }

    pub fn songs(&self) -> &HashMap<String, Song> {
        &self.songs
    }

    pub fn last_played_album_id(&self) -> &str {
        &self.last_played_album_id
    }

    pub fn set_last_played_album_id(&mut self, last_played_album_id: String) {
        self.last_played_album_id = last_played_album_id;
    }

    pub fn contains_artist(&self, artist_id: &str) -> bool {
        self.artists.contains_key(artist_id)
    }
    pub fn insert_artist(&mut self, id: String, artist: Artist) {
        self.artists.insert(id, artist);
    }

    pub fn get_artist_mut(&mut self, id: &str) -> &mut Artist {
        self.artists.get_mut(id).unwrap()
    }

    pub fn get_artist(&self, id: &str) -> &Artist {
        self.artists.get(id).unwrap()
    }

    pub fn artists(&self) -> &HashMap<String, Artist> {
        &self.artists
    }

    pub fn alphabetical_artists(&self) -> &Vec<String> {
        &self.alphabetical_artists
    }

    pub fn set_alphabetical_artists(&mut self, alphabetical_artists: Vec<String>) {
        self.alphabetical_artists = alphabetical_artists;
    }

    pub fn alphabetical_playlists(&self) -> &Vec<String> {
        &self.alphabetical_playlists
    }

    pub fn set_alphabetical_playlists(&mut self, alphabetical_playlists: Vec<String>) {
        self.alphabetical_playlists = alphabetical_playlists;
    }

    pub fn number_of_local_playlists(&self) -> usize {
        self.number_of_local_playlists
    }

    pub fn set_number_of_local_playlists(&mut self, number_of_local_playlists: usize) {
        self.number_of_local_playlists = number_of_local_playlists;
    }
    pub fn get_number_of_albums(&self) -> usize {
        self.albums.len()
    }
    pub fn albums(&self) -> &HashMap<String, Album> {
        &self.albums
    }
    pub fn remove_album(&mut self, id: &str) {
        self.albums.remove(id);
    }
    pub fn favorite_genres(&self) -> &Vec<String> {
        &self.favorite_genres
    }
    pub fn push_favorite_genre(&mut self, genre: String) {
        self.favorite_genres.push(genre);
    }
    pub fn remove_favorite_genre(&mut self, position: usize) {
        self.favorite_genres.remove(position);
    }

    pub fn update_artist(&mut self, artist_id: &str) {
        let artist = match self.artists.get_mut(artist_id) {
            None => {
                warn!("Tried to update artist, but is missing in database: {}", artist_id);
                return;
            }
            Some(value) => value
        };

        artist
            .albums_mut()
            .retain(|album_id| self.albums.contains_key(album_id));

        // If the artist has no more albums, delete it
        if artist.albums().is_empty() {
            info!(
                "Artist {} ({}) has no albums, deleting",
                artist.name(),
                artist_id
            );
            self.artists.remove(artist_id);
            return;
        }

        // Set the updated genres
        let updated_genres: Vec<String> = artist
            .albums()
            .iter()
            .flat_map(|album_id| self.albums.get(album_id).unwrap().genres().iter())
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        artist.set_genres(updated_genres);

        // Set the updated number of albums
        artist.set_number_of_albums(artist.albums().len());
    }

    pub fn update_playlist_dates(&mut self, playlist_id: &str, modified_date: &str) {
        let playlist = self.playlists.get_mut(playlist_id).unwrap();
        playlist.set_modified_on(modified_date.to_string());
    }

    pub fn album_contains_genre(&self, id: &str, genre: &str) -> bool {
        let genre = genre.to_lowercase();

        if let Some(album) = self.albums.get(id) {
            for album_genre in album.genres() {
                if album_genre.to_lowercase().contains(&genre) {
                    return true;
                }
            }
        }

        false
    }
}
