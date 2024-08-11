use zbus::{interface, Connection};
use crate::app::AppResult;

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

// Although we use `tokio` here, you can use any async runtime of choice.
pub async fn set_up_mpris() -> AppResult<Connection> {
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
    // before requesting the name
    connection
        .request_name("org.mpris.MediaPlayer2.naviterm")
        .await?;

    Ok(connection)

}
