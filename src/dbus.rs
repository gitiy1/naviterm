use std::collections::HashMap;
use log::debug;
use tokio::sync::mpsc::UnboundedSender;
use zbus::{interface, Connection};
use zbus::zvariant::{Value, ObjectPath};
use crate::app::AppResult;
use crate::event::{Event};

struct MediaPlayer2 {
    can_quit: bool,
    fullscreen: bool,
    can_set_fullscreen: bool,
    can_raise: bool,
    has_track_list: bool,
    identity: String,
    desktop_entry: String,
    supported_uri_schemes: Vec<String>,
    supported_mime_types: Vec<String>,
}

#[interface(name = "org.mpris.MediaPlayer2")]
impl MediaPlayer2 {

    #[zbus(property)]
    async fn can_quit(&self) -> &bool {
        &self.can_quit
    }

    #[zbus(property)]
    async fn fullscreen(&self) -> &bool {
        &self.fullscreen
    }

    #[zbus(property)]
    async fn set_fullscreen(&mut self, fullscreen: bool) {
        if self.can_set_fullscreen { self.fullscreen = fullscreen }
    }

    #[zbus(property)]
    async fn can_set_fullscreen(&self) -> &bool {
        &self.can_set_fullscreen
    }

    #[zbus(property)]
    async fn can_raise(&self) -> &bool {
        &self.can_raise
    }

    #[zbus(property)]
    async fn has_track_list(&self) -> &bool {
        &self.has_track_list
    }

    #[zbus(property)]
    async fn identity(&self) -> &str {
        &self.identity
    }

    #[zbus(property)]
    async fn desktop_entry(&self) -> &str {
        &self.desktop_entry
    }

    #[zbus(property)]
    async fn supported_uri_schemes(&self) -> &Vec<String> {
        &self.supported_uri_schemes
    }

    #[zbus(property)]
    async fn supported_mime_types(&self) -> &Vec<String> {
        &self.supported_mime_types
    }

    async fn raise(&self) {
        // TODO
    }

    async fn quit(&self) {
        // TODO
    }
}

pub struct MediaPlayer2Player {
    can_play: bool,
    can_pause: bool,
    can_control: bool,
    can_go_next: bool,
    can_go_previous: bool,
    playback_status: String,
    metadata: HashMap<String,String>,
    sender: UnboundedSender<Event>,
}

#[interface(name = "org.mpris.MediaPlayer2.Player")]
impl MediaPlayer2Player {

    #[zbus(property)]
    async fn can_control(&self) -> &bool {
        &self.can_control
    }

    #[zbus(property)]
    async fn can_go_next(&self) -> &bool {
        &self.can_go_next
    }

    #[zbus(property)]
    async fn can_go_previous(&self) -> &bool {
        &self.can_go_previous
    }

    #[zbus(property)]
    async fn can_play(&self) -> &bool {
        &self.can_play
    }

    #[zbus(property)]
    async fn can_pause(&self) -> &bool {
        &self.can_pause
    }

    #[zbus(property)]
    async fn playback_status(&self) -> &str {
        &self.playback_status
    }

    #[zbus(property)]
    async fn metadata(&self) -> HashMap<String, Value> {
        let mut fields = HashMap::new();
        for field in &self.metadata {
            if field.0.starts_with("title") {
                fields.insert("xesam:title".to_string(), Value::from(field.1));
            }
            else if field.0.starts_with("album") {
                fields.insert("xesam:album".to_string(), Value::from(field.1));
            }
            else if field.0.starts_with("artist") {
                let artist_vector = vec![field.1];
                fields.insert("xesam:artist".to_string(), Value::from(artist_vector));
            }
            else if field.0.starts_with("cover") {
                fields.insert("mpris:artUrl".to_string(), Value::from(field.1));
            }
            else if field.0.starts_with("id") {
                let str_path = format!("/org/node/mediaplayer/naviterm/track/{}",field.1 );
                let path = ObjectPath::try_from(str_path).unwrap();
                fields.insert("mpris:trackid".to_string(), Value::from(path));
            }
        }
        fields
    }

    async fn play_pause(&self) {
        debug!("PlayPause request from dbus!\n");
        self.sender.send(Event::PlayPause).unwrap();
    }

    async fn play(&self) {
        debug!("Play request from dbus!\n");
        self.sender.send(Event::Play).unwrap();
    }

    async fn pause(&self) {
        debug!("Pause request from dbus!\n");
        self.sender.send(Event::Pause).unwrap();
    }

    async fn next(&self) {
        debug!("Next request from dbus!\n");
        self.sender.send(Event::Next).unwrap();
    }

    async fn previous(&self) {
        debug!("Previous request from dbus!\n");
        self.sender.send(Event::Previous).unwrap();
    }
    
    async fn stop(&self) {
        debug!("Stop request from dbus!\n");
        self.sender.send(Event::Stop).unwrap();
    }

}

impl MediaPlayer2Player {
    pub fn set_playback_status(&mut self, new_status: String) {
        self.playback_status = new_status;
    }
    
    pub fn set_metadata(&mut self, new_metadata: HashMap<String,String>) {
        self.metadata = new_metadata;
    }
}

// Although we use `tokio` here, you can use any async runtime of choice.
pub async fn set_up_mpris(sender: UnboundedSender<Event>) -> AppResult<Connection> {
    let connection = Connection::session().await?;
    // set up the object server
    connection.object_server().at(
        "/org/mpris/MediaPlayer2",
        MediaPlayer2 {
                can_quit: true,
                fullscreen: false,
                can_set_fullscreen: false,
                can_raise: false,
                has_track_list: false,
                identity: String::from("naviterm"),
                desktop_entry: String::from("/usr/share/applications/naviterm.desktop"),
                supported_mime_types: vec!["audio/mpeg".to_string(), "application/ogg".to_string()],
                supported_uri_schemes: vec!["file".to_string()],
        },
        ).await?;
    connection.object_server().at(
        "/org/mpris/MediaPlayer2",
        MediaPlayer2Player {
                can_play: true,
                can_pause: true,
                can_control: true,
                can_go_next: true,
                can_go_previous: true,
                metadata:HashMap::new(),
                playback_status: String::from("Stopped"),
                sender
        }
    ).await?;
    // before requesting the name
    connection
        .request_name("org.mpris.MediaPlayer2.naviterm")
        .await?;

    Ok(connection)

}
