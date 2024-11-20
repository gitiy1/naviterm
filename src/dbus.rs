use crate::app::AppResult;
use crate::event::DbusEvent::{
    Next, Pause, Play, PlayPause, Previous, SeekBackwards, SeekForward, Stop,
};
use crate::event::{DbusEvent, Event};
use log::debug;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use zbus::zvariant::{ObjectPath, Value};
use zbus::{interface, Connection, SignalContext};

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
        if self.can_set_fullscreen {
            self.fullscreen = fullscreen
        }
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
    shuffle: bool,
    playback_status: String,
    position: i64,
    volume: f64,
    metadata: HashMap<String, String>,
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
    async fn can_seek(&self) -> bool {
        true
    }

    #[zbus(property)]
    async fn playback_status(&self) -> &str {
        &self.playback_status
    }

    #[zbus(property)]
    async fn rate(&self) -> f64 {
        1.0
    }

    #[zbus(property)]
    async fn minimum_rate(&self) -> f64 {
        1.0
    }

    #[zbus(property)]
    async fn maximum_rate(&self) -> f64 {
        1.0
    }

    #[zbus(property)]
    async fn position(&self) -> &i64 {
        &self.position
    }

    #[zbus(property)]
    async fn volume(&self) -> &f64 {
        &self.volume
    }

    #[zbus(property)]
    async fn set_volume(&mut self, volume: f64) {
        debug!("Volume change request from dbus!");
        self.sender
            .send(Event::Dbus(DbusEvent::Volume(volume)))
            .unwrap();
    }

    #[zbus(property)]
    async fn shuffle(&self) -> &bool {
        &self.shuffle
    }

    #[zbus(property)]
    async fn set_shuffle(&mut self, _shuffle: bool) {
        debug!("Shuffle request from dbus!");
        self.sender.send(Event::Dbus(DbusEvent::Shuffle)).unwrap();
    }

    #[zbus(property)]
    async fn metadata(&self) -> HashMap<String, Value> {
        let mut fields = HashMap::new();
        for field in &self.metadata {
            if field.0.starts_with("title") {
                fields.insert("xesam:title".to_string(), Value::from(field.1));
            } else if field.0.starts_with("album") {
                fields.insert("xesam:album".to_string(), Value::from(field.1));
            } else if field.0.starts_with("artist") {
                let artist_vector = vec![field.1];
                fields.insert("xesam:artist".to_string(), Value::from(artist_vector));
            } else if field.0.starts_with("cover") {
                fields.insert("mpris:artUrl".to_string(), Value::from(field.1));
            } else if field.0.starts_with("length") {
                let us_length = field.1.parse::<i64>().unwrap() * 1000000;
                fields.insert("mpris:length".to_string(), Value::from(us_length));
            } else if field.0.starts_with("id") {
                let str_path = format!("/org/node/mediaplayer/naviterm/track/{}", field.1);
                let path = ObjectPath::try_from(str_path).unwrap();
                fields.insert("mpris:trackid".to_string(), Value::from(path));
            }
        }
        fields
    }

    #[zbus(signal)]
    pub async fn seeked(signal_ctxt: &SignalContext<'_>, position: i64) -> zbus::Result<()>;

    async fn play_pause(&self) {
        debug!("PlayPause request from dbus!");
        self.sender.send(Event::Dbus(PlayPause)).unwrap();
    }

    async fn play(&self) {
        debug!("Play request from dbus!");
        self.sender.send(Event::Dbus(Play)).unwrap();
    }

    async fn pause(&self) {
        debug!("Pause request from dbus!");
        self.sender.send(Event::Dbus(Pause)).unwrap();
    }

    async fn next(&self) {
        debug!("Next request from dbus!");
        self.sender.send(Event::Dbus(Next)).unwrap();
    }

    async fn previous(&self) {
        debug!("Previous request from dbus!");
        self.sender.send(Event::Dbus(Previous)).unwrap();
    }

    async fn stop(&self) {
        debug!("Stop request from dbus!");
        self.sender.send(Event::Dbus(Stop)).unwrap();
    }

    async fn seek(&self, offset: i64) {
        debug!("Seek request from dbus!");
        if offset > 0 {
            self.sender.send(Event::Dbus(SeekForward)).unwrap();
        } else {
            self.sender.send(Event::Dbus(SeekBackwards)).unwrap();
        }
    }
}

impl MediaPlayer2Player {
    pub fn set_playback_status(&mut self, new_status: String) {
        self.playback_status = new_status;
    }

    pub fn set_metadata(&mut self, new_metadata: HashMap<String, String>) {
        self.metadata = new_metadata;
    }

    pub fn set_position(&mut self, new_position: i64) {
        self.position = new_position;
    }

    pub fn update_volume(&mut self, new_volume: f64) {
        self.volume = new_volume;
    }
    pub fn update_shuffle(&mut self, new_shuffle_status: bool) {
        self.shuffle = new_shuffle_status;
    }
}

// Although we use `tokio` here, you can use any async runtime of choice.
pub async fn set_up_mpris(sender: UnboundedSender<Event>) -> AppResult<Connection> {
    let connection = Connection::session().await?;
    // set up the object server
    connection
        .object_server()
        .at(
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
        )
        .await?;
    connection
        .object_server()
        .at(
            "/org/mpris/MediaPlayer2",
            MediaPlayer2Player {
                can_play: true,
                can_pause: true,
                can_control: true,
                can_go_next: true,
                can_go_previous: true,
                shuffle: false,
                position: 0,
                volume: 1.0,
                metadata: HashMap::new(),
                playback_status: String::from("Stopped"),
                sender,
            },
        )
        .await?;
    // before requesting the name
    connection
        .request_name("org.mpris.MediaPlayer2.naviterm")
        .await?;

    Ok(connection)
}
