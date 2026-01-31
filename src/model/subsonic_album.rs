use serde::{Deserialize};
use crate::model::subsonic_common::{ArtistRef, NameOnly};
use crate::model::subsonic_song::SongResponse;

#[derive(Debug, Deserialize)]
pub struct AlbumListPayload {
    #[serde(rename = "albumList")]
    pub album_list: AlbumList,
}

#[derive(Debug, Deserialize)]
pub struct AlbumList {
    pub album: Option<Vec<AlbumId>>,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct AlbumId {
    pub id: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct AlbumPayload {
    pub album: AlbumResponse,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct AlbumResponse {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub artist_id: String,
    pub cover_art: String,

    pub song_count: usize,
    pub duration: usize,
    pub play_count: usize,

    pub created: String,
    pub year: usize,

    pub genres: Vec<NameOnly>,
    pub artists: Vec<ArtistRef>,


    #[serde(rename = "song")]
    pub songs: Vec<SongResponse>,
}

