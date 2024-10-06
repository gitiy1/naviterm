use std::collections::HashMap;
use crate::model::album::Album;
use crate::model::song::Song;

#[derive(Debug, Default)]
pub struct MusicDatabase {
    recent_albums: Vec<Album>,
    most_listened_albums: Vec<Album>,
    alphabetical_albums: Vec<Album>,
    filtered_albums: Vec<String>,
    genres: Vec<String>,
    albums: HashMap<String,Album>,
    songs: HashMap<String,Song>,
}

impl MusicDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recent_albums(&self) -> &Vec<Album> {
        &self.recent_albums
    }

    pub fn most_listened_albums(&self) -> &Vec<Album> {
        &self.most_listened_albums
    }

    pub fn alphabetical_list_albums(&self) -> &Vec<Album> {
        &self.alphabetical_albums
    }

    pub fn filtered_albums(&self) -> &Vec<String> {
        &self.filtered_albums
    }

    pub fn set_recent_albums(&mut self, recent_albums: Vec<Album>) {
        self.recent_albums = recent_albums;
    }

    pub fn set_filtered_albums(&mut self, filtered_albums: Vec<String>) {
        self.filtered_albums = filtered_albums;
    }

    pub fn set_most_listened_albums(&mut self, most_listened_albums: Vec<Album>) {
        self.most_listened_albums = most_listened_albums;
    }

    pub fn set_alphabetical_albums(&mut self, alphabetical_list: Vec<Album>) {
        self.alphabetical_albums = alphabetical_list;
    }
    
    pub fn insert_album(&mut self, id: String, album: Album) {
        self.albums.insert(id, album);
    }
    
    pub fn get_album(&self, id: &str) -> &Album {
        self.albums.get(id).unwrap()
    }
    
    pub fn contains_album(&self, id: &str) -> bool {
        self.albums.contains_key(id)
    }

    pub fn insert_song(&mut self, id: String, song: Song) {
        self.songs.insert(id, song);
    }

    pub fn get_song(&self, id: &str) -> &Song {
        self.songs.get(id).unwrap()
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
}