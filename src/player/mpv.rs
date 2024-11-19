use std::process::{Child, Command, Stdio};

use log::debug;

use crate::{app::AppResult, player::ipc::Ipc};

pub const MPV_SOCKET: &str = "/tmp/naviterm_mpv";
pub const MPV_LOG: &str = "/tmp/naviterm_mpv.log";

#[derive(Debug, PartialEq)]
pub enum PlayerStatus {
    Playing,
    Paused,
    Stopped,
}

pub struct Mpv {
    mpv_process: Child,
    pub(crate) player_status: PlayerStatus,
    pub(crate) ipc: Ipc,
    volume: usize
}

impl Default for Mpv {
    fn default() -> Self {
        Self {
            mpv_process: Command::new("mpv")
                .arg("--no-video")
                .arg("--idle")
                .arg("--input-ipc-server=".to_owned() + MPV_SOCKET)
                .arg("--prefetch-playlist=yes")
                .arg("--log-file=".to_owned() + MPV_LOG)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
            player_status: PlayerStatus::Stopped,
            ipc: Ipc::default(),
            volume: 100
        }
    }
}

impl Mpv {
    pub fn initialize(&mut self) -> AppResult<()> {
        self.ipc.initialize_stream()?;
        Ok(())
    }

    pub fn quit_player(&mut self) {
        self.ipc.quit();
        debug!("Message to quit sent, waiting for mpv to exit");
        self.mpv_process
            .wait()
            .expect("Could not wait mpv to finish");
    }

    pub fn play_song(&mut self, song_url: &str) {
        self.ipc.load_file(song_url);
    }

    pub fn add_next_song_to_queue(&mut self, song_url: &str) {
        self.ipc.load_file_next(song_url);
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

    pub fn stop(&mut self) {
        self.ipc.stop();
    }

    pub fn seek_forward(&mut self) {
        self.ipc.seek("10")
    }

    pub fn seek_backwards(&mut self) {
        self.ipc.seek("-10")
    }

    pub fn set_playback_percentage(&mut self, percentage: &str) {
        self.ipc.seek_percentage(percentage);
    }
    
    pub fn set_replay_gain(&mut self, replay_gain_mode: &str) {
        self.ipc.set_replay_gain_mode(replay_gain_mode);
    }
    pub fn player_status(&self) -> &PlayerStatus {
        &self.player_status
    }

    pub async fn poll_ipc_events(&mut self) {
        self.ipc.poll_events().await;
    }
    pub fn get_playback_time(&mut self) -> f64 {
        self.ipc.get_playback_time()
    }
    
    pub fn get_volume(&self) -> usize { self.volume }
    
    pub fn raise_volume(&mut self) {
        if self.volume < 100 {
            self.volume += 2;
            self.ipc.set_volume(self.volume.to_string().as_str());
        }
    }

    pub fn lower_volume(&mut self) {
        if self.volume > 0 {
            self.volume -= 2;
            self.ipc.set_volume(self.volume.to_string().as_str());
        }
    }
}
