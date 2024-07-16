#[derive(Debug,Default)]
pub struct Song {
    id: String,
    track: String,
    title: String,
    album: String,
    album_id: String,
    artist: String,
    artist_id: String,
    cover_art: String,
    duration: String,
    play_count: String,
    genres: Vec<String>,
    album_gain: String,
    album_peak: String,
    track_gain: String,
    track_peak: String,
    bit_rate: String
}

impl Song {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn track(&self) -> &str {
        &self.track
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn album(&self) -> &str {
        &self.album
    }

    pub fn album_id(&self) -> &str {
        &self.album_id
    }

    pub fn artist(&self) -> &str {
        &self.artist
    }

    pub fn artist_id(&self) -> &str {
        &self.artist_id
    }

    pub fn cover_art(&self) -> &str {
        &self.cover_art
    }

    pub fn duration(&self) -> &str {
        &self.duration
    }

    pub fn play_count(&self) -> &str {
        &self.play_count
    }

    pub fn bit_rate(&self) -> &str {
        &self.bit_rate
    }

    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    pub fn set_track(&mut self, track: String) {
        self.track = track;
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_album(&mut self, album: String) {
        self.album = album;
    }

    pub fn set_album_id(&mut self, album_id: String) {
        self.album_id = album_id;
    }

    pub fn set_artist(&mut self, artist: String) {
        self.artist = artist;
    }

    pub fn set_artist_id(&mut self, artist_id: String) {
        self.artist_id = artist_id;
    }

    pub fn set_cover_art(&mut self, cover_art: String) {
        self.cover_art = cover_art;
    }

    pub fn set_duration(&mut self, duration: String) {
        self.duration = duration;
    }

    pub fn set_play_count(&mut self, play_count: String) {
        self.play_count = play_count;
    }

    pub fn set_bit_rate(&mut self, bit_rate: String) {
        self.bit_rate = bit_rate;
    }

    pub fn set_album_gain(&mut self, album_gain: String) {
        self.album_gain = album_gain;
    }

    pub fn set_album_peak(&mut self, album_peak: String) {
        self.album_peak = album_peak;
    }

    pub fn set_track_gain(&mut self, track_gain: String) {
        self.track_gain = track_gain;
    }

    pub fn set_track_peak(&mut self, track_peak: String) {
        self.track_peak = track_peak;
    }

    pub fn genres(&self) -> &Vec<String> {
        &self.genres
    }

    pub fn set_genres(&mut self, genres: Vec<String>) {
        self.genres = genres;
    }

    pub fn album_gain(&self) -> &str {
        &self.album_gain
    }

    pub fn album_peak(&self) -> &str {
        &self.album_peak
    }

    pub fn track_gain(&self) -> &str {
        &self.track_gain
    }

    pub fn track_peak(&self) -> &str {
        &self.track_peak
    }
}
