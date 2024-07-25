use std::process::{Child, Command, Stdio};
use crate::player::ipc::Ipc;

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
    player_status: PlayerStatus,
    ipc: Ipc
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
            player_status: PlayerStatus::Stopped,
            ipc: Ipc::default()
        }
    }
}

impl Mpv {
    
    pub fn initialize(&mut self) {
        self.ipc.initialize_stream();
    }
    
    pub fn quit_player(&mut self) {
        self.ipc.quit();
        self.mpv_process.wait().expect("Could not wait mpv to finish");
    }

    pub fn play_song(&mut self, song_url: &str) {
        self.ipc.load_file(song_url);
        self.player_status = PlayerStatus::Playing;
    }

    pub fn toggle_play_pause(&mut self) {
        match self.player_status {
            PlayerStatus::Playing => {
                self.player_status = PlayerStatus::Paused;
                self.ipc.toggle_play_pause();
            }
            PlayerStatus::Paused => {
                self.player_status = PlayerStatus::Playing;
                self.ipc.toggle_play_pause();
            }
            PlayerStatus::Stopped => {}
        }

    }
    
    pub fn seek_forward(&mut self) {
        self.ipc.seek("10")
    }
    
    pub fn seek_backwards(&mut self) {
        self.ipc.seek("-10")
    }

    pub fn player_status(&self) -> &PlayerStatus {
        &self.player_status
    }
}
