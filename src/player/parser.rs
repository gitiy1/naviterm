use log::{debug, error, warn};
use crate::player::ipc::IpcEvent;

pub fn parse_json_event(event: String) -> Vec<IpcEvent> {
    let mut events = vec![];
    for line in event.lines() {
        let parsed_json = json::parse(line);
        if parsed_json.is_ok() {
            let json_event = parsed_json.unwrap();
            match json_event["event"].as_str() {
                Some(event) => match event {
                    "seek" => events.push(IpcEvent::Seek),
                    "playback-restart" => events.push(IpcEvent::PlaybackRestart),
                    "file-loaded" => events.push(IpcEvent::FileLoaded),
                    "property-change" => {
                        let name = json_event["name"].to_string();
                        let data = json_event["data"].to_string();
                        events.push(IpcEvent::PropertyChange(name,data));
                    }
                    "end-file" => {
                        let reason = json_event["reason"].to_string();
                        events.push(IpcEvent::Eof(reason));
                    }
                    "idle" => events.push(IpcEvent::Idle),
                    _ => {}
                },
                None => {
                    debug!("Line {} does not contain a json event", line);
                    return vec![];
                }
            }
        } else {
            events.push(IpcEvent::Error(String::from(line)))
        }
    }
    events
}

pub fn parse_json_data(response: &str) -> String {
    let parsed_json = json::parse(response);
    if parsed_json.is_ok() {
        let json_data = parsed_json.unwrap();
        json_data["data"].to_string()
    } else {
        String::from("error")
    }
}

pub fn parse_json_success(response: &str) -> bool {
    let parsed_json = json::parse(response);
    if parsed_json.is_ok() {
        let json_data = parsed_json.unwrap();
        match json_data["error"].as_str() {
            None => {
                error!("Could not parse json success response");
                false
            }
            Some(value) => match value {
                "success" => true,
                &_ => {
                    warn!("Got unexpected JSON error: {}", value);
                    false
                },
            }
        }
    } else {
        false
    }
}
