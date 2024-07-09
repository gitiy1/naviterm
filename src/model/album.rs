#[derive(Debug,Default)]
pub struct Album {
    id: String,
    name: String,
    cover_art: String,
    duration: String,
    artist: String,
    genre: String,
    song_count: String
}

impl Album {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn cover_art(&self) -> &str {
        &self.cover_art
    }

    pub fn duration(&self) -> &str {
        &self.duration
    }

    pub fn artist(&self) -> &str {
        &self.artist
    }

    pub fn song_count(&self) -> &str {
        &self.song_count
    }

    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_cover_art(&mut self, cover_art: String) {
        self.cover_art = cover_art;
    }

    pub fn set_duration(&mut self, duration: String) {
        self.duration = duration;
    }

    pub fn set_artist(&mut self, artist: String) {
        self.artist = artist;
    }

    pub fn set_song_count(&mut self, song_count: String) {
        self.song_count = song_count;
    }

    pub fn genre(&self) -> &str {
        &self.genre
    }

    pub fn set_genre(&mut self, genre: String) {
        self.genre = genre;
    }
}


