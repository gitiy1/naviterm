use std::process::{Child, Command, Stdio};
use crate::player::ipc;

pub const MPV_SOCKET: &str = "/tmp/naviterm_mpv";

#[derive(Debug)]
pub enum PlayerStatus {
    Playing,
    Paused,
    Stopped
}

#[derive(Debug)]
pub struct Mpv {
    mpv_process: Child,
    player_status: PlayerStatus
}

impl Default for Mpv {
    fn default() -> Self {
        Self {
            mpv_process: Command::new("mpv")
                .arg("--no-video").arg("--idle").arg("--input-ipc-server=".to_owned()+MPV_SOCKET)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn().unwrap(),
            player_status: PlayerStatus::Stopped
        }
    }
}

impl Mpv {
    pub fn quit_player(&mut self) {
        ipc::quit();
        self.mpv_process.wait().expect("Could not wait mpv to finish");
    }

    pub fn play_song(&mut self, song_url: &str) {
        ipc::load_file(song_url);
        self.player_status = PlayerStatus::Playing;
    }

    pub fn toggle_play_pause(&mut self) {
        match self.player_status {
            PlayerStatus::Playing => {
                self.player_status = PlayerStatus::Paused;
                ipc::toggle_play_pause();
            }
            PlayerStatus::Paused => {
                self.player_status = PlayerStatus::Playing;
                ipc::toggle_play_pause();
            }
            PlayerStatus::Stopped => {}
        }

    }

    pub fn player_status(&self) -> &PlayerStatus {
        &self.player_status
    }
}
