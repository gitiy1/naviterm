use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use crate::model::subsonic_song::SongResponse;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct PlaylistsPayload {
    pub playlists: Playlists,
}
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct PlaylistPayload {
    pub playlist: PlaylistResponse,
}
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Playlists {
    pub playlist: Vec<PlaylistResponse>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct PlaylistResponse {
    pub id: String,
    pub name: String,
    pub comment: String,
    pub song_count: usize,
    pub duration: usize,
    pub public: bool,
    pub owner: String,
    pub created: DateTime<FixedOffset>,
    pub changed: DateTime<FixedOffset>,
    pub entry: Option<Vec<SongResponse>>
}
