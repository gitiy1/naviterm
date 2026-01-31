use serde::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct SubsonicEnvelope<T> {
    #[serde(rename = "subsonic-response")]
    pub subsonic_response: SubsonicResponse<T>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "status")]
pub enum SubsonicResponse<T> {
    #[serde(rename = "ok")]
    Ok {
        version: String,
        #[serde(rename = "type")]
        server_type: String,
        #[serde(rename = "serverVersion")]
        server_version: Option<String>,
        #[serde(rename = "openSubsonic")]
        open_subsonic: Option<bool>,
        #[serde(flatten)]
        payload: T,
    },

    #[serde(rename = "failed")]
    Failed {
        version: String,
        #[serde(rename = "type")]
        server_type: String,
        #[serde(rename = "serverVersion")]
        server_version: Option<String>,
        #[serde(rename = "openSubsonic")]
        open_subsonic: Option<bool>,
        error: SubsonicError,
    },
}

#[derive(Debug, Deserialize)]
pub struct EmptyPayload { }
#[derive(Debug, Deserialize)]
pub struct SubsonicError {
    pub code: usize,
    pub message: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct ArtistRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct NameOnly {
    pub name: String,
}
