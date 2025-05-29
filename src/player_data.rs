use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub enum AppLoopStatus {
    #[default]
    None,
    Track,
    Playlist,
}

impl AppLoopStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppLoopStatus::None => "off",
            AppLoopStatus::Track => "track",
            AppLoopStatus::Playlist => "playlist",
        }
    }
    
}

#[derive(Serialize, Deserialize, Default)]
pub struct NowPlaying {
    pub id: String,
    pub duration: String,
}


#[derive(Serialize, Deserialize, Default)]
pub struct PlayerData {
    pub duration_total: String,
    pub duration_left: String,
    pub queue: Vec<String>,
    pub queue_order: Vec<usize>,
    pub now_playing: NowPlaying,
    pub index_in_queue: usize,
    pub random_playback: bool,
    pub next_is_in_player_queue: bool,
    pub loop_status: AppLoopStatus,
    pub player_volume: usize
}

