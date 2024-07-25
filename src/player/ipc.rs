use std::io::Write;
use std::os::unix::net::UnixStream;

#[derive(Debug, Default)]
pub struct Ipc {
    stream: Option<UnixStream>
}

impl Ipc {
    pub fn initialize_stream(&mut self) {
        self.stream = Some(UnixStream::connect("/tmp/naviterm_mpv").expect("Cannot create ipc stream"));
    }

    pub fn load_file(&self, file_url: &str) {
        let msg = r#"{"command":["loadfile", ""#.to_owned() + file_url + r#""]}"# + "\n";
        self.stream.as_ref().unwrap()
            .write_all(msg.as_bytes())
            .expect("ipc: Error while loading file");
    }

    pub fn quit(&self) {
        self.stream.as_ref().unwrap()
            .write_all(b"{\"command\":[\"quit\"]}\n")
            .expect("ipc: Error while exiting ipc connection");
    }

    pub fn toggle_play_pause(&self) {
        self.stream.as_ref().unwrap()
            .write_all(b"{\"command\":[\"cycle\",\"pause\"]}\n")
            .expect("ipc: Error while cycling pause");
    }
}
