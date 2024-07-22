use std::process::{Child, Command, Stdio};
use crate::player::ipc;

pub const MPV_SOCKET: &str = "/tmp/naviterm_mpv";

#[derive(Debug)]
pub struct Mpv {
    mpv_process: Child,
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
        }
    }
}

impl Mpv {
    pub fn quit_player(&mut self) {
        ipc::quit();
        self.mpv_process.wait().expect("Could not wait mpv to finish");
    }
}

pub fn play_song(song_url: &str) {
    ipc::load_file(song_url);   
}