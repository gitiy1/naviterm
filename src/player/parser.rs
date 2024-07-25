use crate::player::ipc::{IpcEvent};

pub fn parse_json_event(event: String) -> IpcEvent {
   IpcEvent::Idle
}