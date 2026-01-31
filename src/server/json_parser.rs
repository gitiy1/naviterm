use crate::app::AppResult;
use crate::model::album::Album;
use crate::model::artist::Artist;
use crate::model::connection_status::ConnectionStatus;
use crate::model::playlist::Playlist;
use crate::model::song::Song;
use crate::model::subsonic_album::{AlbumListPayload, AlbumPayload};
use crate::model::subsonic_common::{EmptyPayload, SubsonicEnvelope, SubsonicResponse};
use crate::model::subsonic_genre::GenresPayload;
use crate::model::subsonic_playlist::{PlaylistPayload, PlaylistsPayload};

pub struct JsonParser {}

impl JsonParser {
    pub fn parse_connection_status(response: String) -> AppResult<ConnectionStatus> {
        let mut connection_status: ConnectionStatus = ConnectionStatus::default();
        let envelope: SubsonicEnvelope<EmptyPayload> = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse ping JSON: {e}"))?;

        match envelope.subsonic_response {
            SubsonicResponse::Ok { version, .. } => {
                connection_status.set_status("Ok".to_string());
                connection_status.set_server_version(version.to_string());
            }
            SubsonicResponse::Failed { error, version, .. } => {
                connection_status.set_status("Failed".to_string());
                connection_status.set_server_version(version.to_string());
                connection_status.set_error_code(error.code.to_string());
                connection_status.set_error_code(error.message.to_string());
            }
        }

        Ok(connection_status)
    }

    pub fn parse_genres_list(response: String) -> AppResult<Vec<String>> {
        let envelope: SubsonicEnvelope<GenresPayload> = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse genres list JSON: {e}"))?;

        match envelope.subsonic_response {
            SubsonicResponse::Ok { payload, .. } => Ok(payload
                .genres
                .genre
                .iter()
                .map(|genre| genre.value.clone())
                .collect()),
            SubsonicResponse::Failed { error, .. } => Err(format!(
                "Error parsing response for genres list: {}-{}",
                error.code, error.message
            )
            .into()),
        }
    }

    pub fn parse_album_list_simple(response: String, _api_version: &str) -> AppResult<Vec<String>> {
        let envelope: SubsonicEnvelope<AlbumListPayload> = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse album list JSON: {e}"))?;

        match envelope.subsonic_response {
            SubsonicResponse::Ok { payload, .. } => {
                if payload.album_list.album.is_none() {
                    Ok(Vec::new())
                } else {
                    Ok(payload
                        .album_list
                        .album
                        .unwrap()
                        .iter()
                        .map(|album_id| album_id.id.clone())
                        .collect())
                }
            }
            SubsonicResponse::Failed { error, .. } => Err(format!(
                "Error parsing response for album list: {}-{}",
                error.code, error.message
            )
            .into()),
        }
    }

    pub fn parse_album(response: String) -> AppResult<(Album, Vec<Song>, Artist)> {
        let envelope: SubsonicEnvelope<AlbumPayload> = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse album JSON: {e}"))?;

        match envelope.subsonic_response {
            SubsonicResponse::Ok { payload, .. } => {
                let album: Album = (&payload.album).into();
                let songs: Vec<Song> = payload.album.songs.iter().map(|song| song.into()).collect();
                let artist: Artist = (&payload.album).into();
                Ok((album, songs, artist))
            }
            SubsonicResponse::Failed { error, .. } => Err(format!(
                "Error parsing response for album: {}-{}",
                error.code, error.message
            )
            .into()),
        }
    }

    pub fn parse_playlist_list(response: String) -> AppResult<Vec<Playlist>> {
        let envelope: SubsonicEnvelope<PlaylistsPayload> = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse playlist JSON: {e}"))?;

        match envelope.subsonic_response {
            SubsonicResponse::Ok { payload, .. } => Ok(payload
                .playlists
                .playlist
                .iter()
                .map(|playlist| playlist.into())
                .collect()),
            SubsonicResponse::Failed { error, .. } => Err(format!(
                "Error parsing response for playlist list: {}-{}",
                error.code, error.message
            )
            .into()),
        }
    }

    pub fn parse_playlist(response: String) -> AppResult<Vec<String>> {
        let envelope: SubsonicEnvelope<PlaylistPayload> = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse playlist JSON: {e}"))?;

        match envelope.subsonic_response {
            SubsonicResponse::Ok { payload, .. } => {
                if let Some(song_list) = payload.playlist.entry {
                    Ok(song_list.iter().map(|song| song.id.clone()).collect())
                } else {
                    Err("Error parsing song list playlist".to_string().into())
                }
            }
            SubsonicResponse::Failed { error, .. } => Err(format!(
                "Error parsing response for playlist list: {}-{}",
                error.code, error.message
            )
            .into()),
        }
    }

    pub fn parse_playlist_id(response: String) -> AppResult<String> {
        let envelope: SubsonicEnvelope<PlaylistPayload> = serde_json::from_str(&response)
            .map_err(|e| format!("Failed to parse playlist JSON: {e}"))?;

        match envelope.subsonic_response {
            SubsonicResponse::Ok { payload, .. } => Ok(payload.playlist.id),
            SubsonicResponse::Failed { error, .. } => Err(format!(
                "Error parsing response for playlist list: {}-{}",
                error.code, error.message
            )
            .into()),
        }
    }
}
