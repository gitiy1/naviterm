use crate::player::ipc::{IpcEvent};

pub fn parse_json_event(event: String) -> Vec<IpcEvent >{
   let mut events = vec![];
   for line in event.lines() {
      let parsed_json = json::parse(line);
      if parsed_json.is_ok() {
         let json_event = parsed_json.unwrap();
         match json_event["event"].as_str().unwrap() {
            "seek" => {
               events.push(IpcEvent::Seek)
            },
            "file-loaded" => {
               events.push(IpcEvent::FileLoaded)
            },
            "end-file" => {
               let reason = json_event["reason"].to_string();
               events.push(IpcEvent::Eof(reason));
            }
            "idle" => {
               events.push(IpcEvent::Idle)
            },
            _ => {}
         }
      } else {
         events.push(IpcEvent::Error(String::from(line)))
      }
   }
   events
}