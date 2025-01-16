use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Playlist {
    id: String,
    name: String,
    song_count: String,
    duration: String,
    modified: bool,
    song_list: Vec<String>,
    created_on: String,
    modified_on: String,
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

    pub fn song_list_mut(&mut self) -> &mut Vec<String> {
        &mut self.song_list
    }

    pub fn set_song_list(&mut self, song_list: Vec<String>) {
        self.song_list = song_list;
    }
    
    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    pub fn modified(&self) -> bool {
        self.modified
    }

    pub fn created_on(&self) -> &str {
        &self.created_on
    }

    pub fn modified_on(&self) -> &str {
        &self.modified_on
    }

    pub fn set_modified_on(&mut self, modified_on: String) {
        self.modified_on = modified_on;
    }

    pub fn set_created_on(&mut self, created_on: String) {
        self.created_on = created_on;
    }
}