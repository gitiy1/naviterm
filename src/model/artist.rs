use serde::{Deserialize, Serialize};
use crate::model::subsonic_album::AlbumResponse;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Artist {
    id: String,
    name: String,
    albums: Vec<String>,
    genres: Vec<String>,
}

impl Artist {
    pub fn id(&self) -> &str {
        &self.id
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
        self.albums.push(album);
        for genre in new_genres {
            if !self.genres.contains(&genre) {
                self.genres.push(genre);
            }
        }
    }
}

impl From<&AlbumResponse> for Artist {
    fn from(album: &AlbumResponse) -> Self {
        Self {
            id: album.artist_id.to_string(),
            name: album.artist.to_string(),
            albums: vec![],
            genres: album.genres.iter().map(|genre| genre.name.clone()).collect(),
        }
    }
}
