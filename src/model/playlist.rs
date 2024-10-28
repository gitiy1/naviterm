use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Playlist {
    id: String,
    name: String,
    song_count: String,
    duration: String,
    song_list: Vec<String>
}

impl Playlist {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn song_count(&self) -> &str {
        &self.song_count
    }

    pub fn duration(&self) -> &str {
        &self.duration
    }

    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_song_count(&mut self, song_count: String) {
        self.song_count = song_count;
    }

    pub fn set_duration(&mut self, duration: String) {
        self.duration = duration;
    }

    pub fn song_list(&self) -> &Vec<String> {
        &self.song_list
    }

    pub fn set_song_list(&mut self, song_list: Vec<String>) {
        self.song_list = song_list;
    }
}