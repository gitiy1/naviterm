use crate::model::album::Album;

#[derive(Debug)]
pub struct MusicDatabase {
    recent_albums: Vec<Album>,
}

impl Default for MusicDatabase {
    fn default() -> Self {
        Self {
            recent_albums: vec![],
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
}