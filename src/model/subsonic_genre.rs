use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct GenresPayload {
    pub genres: GenresResponse,
}


#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct GenresResponse {
    pub genre: Vec<GenreListValue>
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct GenreListValue {
    pub value: String,
    pub song_count: usize,
    pub album_count: usize

}
