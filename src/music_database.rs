use crate::model::album::Album;
use crate::model::song::Song;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::model::artist::Artist;
use crate::model::playlist::Playlist;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MusicDatabase {
    recent_albums: Vec<String>,
    most_listened_albums: Vec<String>,
    alphabetical_albums: Vec<String>,
    most_listened_tracks: Vec<String>,
    recently_added_albums: Vec<String>,
    filtered_albums: Vec<String>,
    genres: Vec<String>,
    albums: HashMap<String, Album>,
    songs: HashMap<String, Song>,
    playlists: HashMap<String,Playlist>,
    artists: HashMap<String,Artist>,
    alphabetical_artists: Vec<String>,
    alphabetical_playlists: Vec<String>,
    last_played_album_id: String,
    number_of_local_playlists: usize,
}

impl MusicDatabase {
    pub fn new() -> Self {
        Self::default()
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

    pub fn alphabetical_list_albums(&self) -> &Vec<String> {
        &self.alphabetical_albums
    }

    pub fn filtered_albums(&self) -> &Vec<String> {
        &self.filtered_albums
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
        self.albums.get(id).unwrap()
    }

    pub fn get_album_mut(&mut self, id: &str) -> &Album {
        self.albums.get(id).unwrap()
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
        self.songs.get(id).unwrap()
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
}
