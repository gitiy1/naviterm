use std::collections::HashMap;
use crate::model::album::Album;

#[derive(Debug)]
pub struct MusicDatabase {
    recent_albums: Vec<Album>,
    albums: HashMap<String,Album>,
}

impl Default for MusicDatabase {
    fn default() -> Self {
        Self {
            recent_albums: vec![],
            albums: HashMap::new()
        }
    }
}

impl MusicDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recent_albums(&self) -> &Vec<Album> {
        &self.recent_albums
    }

    pub fn set_recent_albums(&mut self, recent_albums: Vec<Album>) {
        self.recent_albums = recent_albums;
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
}