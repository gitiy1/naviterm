use crate::app::AppResult;
use crate::player::parser::{parse_json_data, parse_json_event, parse_json_success};
use log::{debug, error, warn};
use num_traits::Num;
use std::f64;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream as OtherUnixStream;

#[derive(Debug)]
pub enum IpcEvent {
    FileLoaded,
    Eof(String),
    PropertyChange(String,String),
    Seek,
    PlaybackRestart,
    Idle,
    Error(String),
    Unrecognized(String),
}

#[derive(Default)]
pub struct Ipc {
    stream: Option<UnixStream>,
    events: Arc<Mutex<Vec<IpcEvent>>>,
    parsed_value: String,
}

impl Ipc {
    pub fn initialize_stream(&mut self) -> AppResult<()> {
        self.stream = Some(UnixStream::connect("/tmp/naviterm_mpv")?);
        Ok(())
    }

    pub fn load_file(&mut self, file_url: &str) {
        let msg = r#"{"command":["loadfile", ""#.to_owned() + file_url + r#""]}"# + "\n";
        self.send_ipc_command(msg, false);
    }

    pub fn load_file_next(&mut self, file_url: &str) {
        let msg =
            r#"{"command":["loadfile", ""#.to_owned() + file_url + r#"", "insert-next"]}"# + "\n";
        self.send_ipc_command(msg, false);
    }

    pub fn quit(&mut self) {
        let msg = String::from("{\"command\":[\"quit\"]}\n");
        self.send_ipc_command(msg, false);
    }

    pub fn toggle_play_pause(&mut self) {
        let msg = String::from("{\"command\":[\"cycle\",\"pause\"]}\n");
        self.send_ipc_command(msg, false);
    }

    pub fn seek(&mut self, amount: &str) {
        let msg = "{\"command\":[\"seek\",\"".to_owned() + amount + "\"]}\n";
        debug!("Sending command to seek");
        self.send_ipc_command(msg, false);
    }

    pub fn seek_percentage(&mut self, percentage: &str) {
        let msg =
            "{\"command\":[\"seek\",\"".to_owned() + percentage + "\",\"absolute-percent\"]}\n";
        debug!("Sending command to seek absolute percent: {}%", percentage);
        self.send_ipc_command(msg, false);
    }

    pub fn stop(&mut self) {
        let msg = String::from("{\"command\":[\"stop\"]}\n");
        debug!("Sending command to stop playback");
        self.send_ipc_command(msg, false);
    }

    pub fn set_replay_gain_mode(&mut self, mode: &str) {
        let msg = "{\"command\":[\"set_property\",\"replaygain\",\"".to_owned() + mode + "\"]}\n";
        debug!("Sending command to set replay-gain mode: {}", mode);
        self.send_ipc_command(msg, false);
    }

    pub fn set_volume(&mut self, volume: &str) {
        let msg = "{\"command\":[\"set_property\",\"volume\",\"".to_owned() + volume + "\"]}\n";
        debug!("Sending command to set volume: {}", volume);
        self.send_ipc_command(msg, false);
    }

    pub fn set_pause(&mut self, paused: &str) {
        let msg = "{\"command\":[\"set_property\",\"pause\",".to_owned() + paused + "]}\n";
        debug!("Sending command to set pause status: {}", paused);
        self.send_ipc_command(msg, false);
    }

    pub fn set_loop_mode(&mut self, loop_mode: &str) {
        let msg = "{\"command\":[\"set_property\",\"loop-file\",\"".to_owned() + loop_mode + "\"]}\n";
        debug!("Sending command to set loop on file: {}", loop_mode);
        self.send_ipc_command(msg, false);
    }

    pub fn get_playback_time(&mut self) -> f64 {
        let msg = String::from("{\"command\":[\"get_property_string\",\"playback-time\"]}\n");
        debug!("Sending command to get playback-time");
        self.parsed_value.clear();
        self.send_ipc_command(msg, true);
        if !self.parsed_value.is_empty() {
            f64::from_str_radix(self.parsed_value.as_str(), 10).unwrap_or_else(|e| {
                error!("Error while parsing response from mpv: {}", e);
                -1.0
            })
        } else {
            error!("Could not get playback time from response received from mpv");
            -1.0
        }
    }

    pub fn get_duration(&mut self) -> f64 {
        let msg = String::from("{\"command\":[\"get_property_string\",\"duration\"]}\n");
        debug!("Sending command to get duration");
        self.parsed_value.clear();
        self.send_ipc_command(msg, true);
        if !self.parsed_value.is_empty() {
            f64::from_str_radix(self.parsed_value.as_str(), 10).unwrap_or_else(|e| {
                error!("Error while parsing response from mpv: {}", e);
                -1.0
            })
        } else {
            error!("Could not get playback time from response received from mpv");
            -1.0
        }
    }
    
    pub async fn poll_events(&mut self) {
        let events = self.events.clone();

        tokio::spawn(async move {
            let mut tokio_stream = OtherUnixStream::connect("/tmp/naviterm_mpv").await.unwrap();
            // Give some time before writing
            tokio::time::sleep(Duration::from_millis(200)).await;

            // Observe pause status changes
            let msg = String::from("{\"command\":[\"observe_property_string\", 0, \"pause\"]}\n");
            let _ = tokio_stream.write_all(msg.as_bytes()).await;
            let mut buf = [0; 4096];
            
            // Read loop
            loop {
                match tokio_stream.read(&mut buf).await {
                    Ok(0) => {
                        // EOF
                        debug!("Socket closed");
                        break;
                    }
                    Ok(n) => {
                        let buf_string = match String::from_utf8(buf[..n].to_vec()) {
                            Ok(s) => s,
                            Err(e) => {
                                error!("Invalid UTF-8: {}", e);
                                continue;
                            }
                        };

                        debug!("Received message: {}", buf_string);
                        let parsed_events = parse_json_event(buf_string);
                        match events.lock() {
                            Ok(mut ev) => ev.extend(parsed_events),
                            Err(e) => error!("Mutex poisoned: {}", e),
                        }
                    }
                    Err(e) => {
                        error!("Read error: {:?}", e);
                        break;
                    }
                }
            }
        });
    }

    pub fn events(&self) -> &Arc<Mutex<Vec<IpcEvent>>> {
        &self.events
    }

    fn read_stream_response(&mut self, parse_response_data: bool) {
        let mut buf = [0; 4096];
        match self.stream.as_ref().unwrap().read(&mut buf) {
            Ok(n) => {
                let buf_string = String::from_utf8(buf[0..n].to_vec()).unwrap();
                debug!("Response from stream: {}", buf_string.trim());
                if parse_response_data {
                    debug!("Parsing data");
                    let response = buf_string.split("\n");
                    for line in response {
                        if parse_json_success(line) {
                            self.parsed_value = parse_json_data(line);
                            debug!("Parsed value: {}", self.parsed_value);
                            return;
                        } else {
                            warn!("Could not parse response from line: {}", line);
                        }
                    }
                   warn!("Could not parse response from stream: {}", buf_string); 
                }
            }
            Err(e) => error!("Failed to read from stream: {}", e),
        }
    }

    fn send_ipc_command(&mut self, msg: String, parse_response_data: bool) {
        debug!(
            "Sending message: {}, parse_response:{}",
            msg.trim(),
            parse_response_data
        );
        match self.stream.as_ref() {
            Some(mut stream) => match stream.write_all(msg.as_bytes()) {
                Ok(_) => {
                    sleep(Duration::from_millis(5));
                    self.read_stream_response(parse_response_data);
                }
                Err(e) => error!("Failed to write to stream: {}", e),
            },
            None => error!("Stream to MPV has not been initialized"),
        }
    }
}
