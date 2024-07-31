use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use log::{debug, error};
use tokio::io;
use tokio::net::{UnixStream as OtherUnixStream};
use crate::player::parser::parse_json_event;

pub enum IpcEvent {
    FileLoaded,
    Eof(String),
    Seek,
    Idle,
    Error(String),
    Unrecognized(String)
}

#[derive(Default)]
pub struct Ipc {
    stream: Option<UnixStream>,
    events: Arc<Mutex<Vec<IpcEvent>>>
}

impl Ipc {
    pub fn initialize_stream(&mut self) {
        self.stream = Some(UnixStream::connect("/tmp/naviterm_mpv").expect("Cannot create ipc stream"));
    }

    pub fn load_file(&self, file_url: &str) {
        let msg = r#"{"command":["loadfile", ""#.to_owned() + file_url + r#""]}"# + "\n";
        self.send_ipc_command(msg);
    }

    pub fn quit(&self) {
        let msg = String::from("{\"command\":[\"quit\"]}\n");
        self.send_ipc_command(msg);
    }

    pub fn toggle_play_pause(&self) {
        let msg = String::from("{\"command\":[\"cycle\",\"pause\"]}\n");
        self.send_ipc_command(msg);
    }

    pub fn seek(&self, amount: &str) {
        let msg = "{\"command\":[\"seek\",\"".to_owned() + amount + "\"]}\n";
        self.send_ipc_command(msg);
    }

    pub fn stop(&self) {
        let msg = String::from("{\"command\":[\"stop\"]}\n");
        self.send_ipc_command(msg);
    }

    pub async fn poll_events(&mut self) {

        let events = self.events.clone();

        tokio::spawn(async move {
            let tokio_stream = OtherUnixStream::connect("/tmp/naviterm_mpv").await.unwrap();
            loop {
                // Wait for the socket to be readable
                tokio_stream.readable().await.unwrap();

                // Creating the buffer **after** the `await` prevents it from
                // being stored in the async task.
                let mut buf = [0; 4096];

                // Try to read data, this may still fail with `WouldBlock`
                // if the readiness event is a false positive.
                match tokio_stream.try_read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let buf_string = String::from_utf8(buf[0..n].to_vec()).unwrap();
                        debug!("Received message: {}", buf_string);
                        let parsed_events = parse_json_event(buf_string);
                        let mut events = events.lock().unwrap();
                        for event in parsed_events {
                            events.push(event);
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    _ => {}
                }
            }
        });
    }

    pub fn events(&self) -> &Arc<Mutex<Vec<IpcEvent>>> {
        &self.events
    }

    fn read_stream_response(&self) {
        let mut buf = [0; 4096];
        match self.stream.as_ref().unwrap().read(&mut buf) {
            Ok(n) => {
                let buf_string = String::from_utf8(buf[0..n].to_vec()).unwrap();
                debug!("Response from stream: {}", buf_string);
            }
            Err(e) => error!("Failed to read from stream: {}", e), }
    }

    fn send_ipc_command(&self, msg: String) {
        match self.stream.as_ref() {
            Some(mut stream) => {
                match stream.write_all(msg.as_bytes()) {
                    Ok(_) => {
                        self.read_stream_response();
                    }
                    Err(e) => error!("Failed to write to stream: {}", e), }
            }
            None => error!("Stream to MPV has not been initialized")
        }
    }
}

