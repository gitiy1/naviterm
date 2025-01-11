use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Artist {
    id: String,
    name: String,
    number_of_albums: usize,
    albums: Vec<String>,
    genres: Vec<String>,
}

impl Artist {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn number_of_albums(&self) -> usize {
        self.number_of_albums
    }

    pub fn albums(&self) -> &Vec<String> {
        &self.albums
    }

    pub fn albums_mut(&mut self) -> &mut Vec<String> {
        &mut self.albums
    }
    pub fn genres(&self) -> &Vec<String> {
        &self.genres
    }

    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    pub fn set_number_of_albums(&mut self, number_of_albums: usize) {
        self.number_of_albums = number_of_albums;
    }

    pub fn set_albums(&mut self, albums: Vec<String>) {
        self.albums = albums;
    }

    pub fn set_genres(&mut self, genres: Vec<String>) {
        self.genres = genres;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, artist_name: String) {
        self.name = artist_name;
    }

    pub fn insert_album(&mut self, album: String, new_genres: Vec<String>) {
        self.number_of_albums += 1;
        self.albums.push(album);
        for genre in new_genres {
            if !self.genres.contains(&genre) {
                self.genres.push(genre);
            }
        }
    }
}
