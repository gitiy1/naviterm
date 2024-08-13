use naviterm::app::{App, AppResult};
use naviterm::event::{Event, EventHandler};
use naviterm::handler::handle_key_events;
use naviterm::tui::Tui;
use std::io;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use config::{Config, ConfigError};
use log4rs::append::file::FileAppender;
use log4rs::Config as log4rsConfig;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::LevelFilter;
use naviterm::dbus;
use naviterm::player::mpv::PlayerStatus;

#[tokio::main]
async fn main() -> AppResult<()> {
    //Load config
    let home_dir = dirs::home_dir().unwrap();
    let mut xdg_conf = home_dir.clone();
    xdg_conf.push(".config/naviterm/config.ini");
    let settings = Config::builder()
        .add_source(config::File::with_name(xdg_conf.to_str().unwrap()))
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();

    let debug_level: Result<String,ConfigError> = settings.get("debug");
    let level: LevelFilter = match debug_level {
        Ok(level) => match level.as_str() {
            "DEBUG" => {
                LevelFilter::Debug
            }
            _ => {
                LevelFilter::Error
            }
        },
        Err(_) => {
            LevelFilter::Error
        }
    };
    let file_path = "/tmp/foo.log";

    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new("{h({d(%+)(utc)} [{f}:{L}] {l:<6} {m})}")))
        .build(file_path)
        .unwrap();

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = log4rsConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(level))
        .unwrap();
    let _handle = log4rs::init_config(config);
    
    // Create an application.
    let mut app = App::new();
    app.set_config(settings)?;
    app.renew_credentials()?;
    app.test_connection().await?;
    app.populate_db().await?;
    app.initialize_player_stream()?;
    app.poll_player_events().await?;
    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    app.set_event_handler(events.sender.clone()).await?;
    let dbus_connection = dbus::set_up_mpris(events.sender.clone()).await?;
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    let iface_ref = dbus_connection
        .object_server()
        .interface::<_, dbus::MediaPlayer2Player>("/org/mpris/MediaPlayer2").await?;
    
    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app).await?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::PlayPause => {
                app.toggle_playing_status().unwrap();
                let mut iface = iface_ref.get_mut().await;
                if *app.player.player_status() == PlayerStatus::Playing {
                    iface.set_playback_status(String::from("Playing"));
                }
                else if *app.player.player_status() == PlayerStatus::Paused {
                    iface.set_playback_status(String::from("Paused"));
                }
                iface.playback_status_changed(iface_ref.signal_context()).await?;
            }
            Event::Next => {app.play_next()?}
            Event::Previous => {app.play_previous()?}
            Event::Playing => {
                let mut iface = iface_ref.get_mut().await;
                iface.set_metadata(app.get_metada_for_current_song());
                iface.metadata_changed(iface_ref.signal_context()).await?;
                iface.set_playback_status(String::from("Playing"));
                iface.playback_status_changed(iface_ref.signal_context()).await?;
            }
            Event::Play => {
                let mut iface = iface_ref.get_mut().await;
                if app.try_play_current() {
                    iface.set_playback_status(String::from("Playing"));
                    iface.playback_status_changed(iface_ref.signal_context()).await?;
                }
            }
            Event::Pause => {
                let mut iface = iface_ref.get_mut().await;
                if app.try_pause_current() {
                    iface.set_playback_status(String::from("Paused"));
                    iface.playback_status_changed(iface_ref.signal_context()).await?;
                }
            }
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
