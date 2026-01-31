use crate::model::subsonic_common::{ArtistRef, NameOnly};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct SongResponse {
    pub id: String,
    pub parent: String,
    pub title: String,
    pub album: String,
    pub artist: String,
    pub cover_art: String,
    pub track: usize,
    pub year: usize,
    pub duration: usize,
    pub bit_rate: usize,
    pub album_id: String,
    pub artist_id: String,
    pub play_count: usize,
    pub genres: Vec<NameOnly>,
    pub artists: Vec<ArtistRef>,
    pub display_artist: String,
    pub replay_gain: ReplayGain,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ReplayGain {
    pub track_gain: f32,
    pub album_gain: f32,
    pub track_peak: f32,
    pub album_peak: f32,
}
