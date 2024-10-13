use naviterm::app::{App, AppResult};
use naviterm::event::{Event, EventHandler};
use naviterm::handler::{handle_dbus_events, handle_key_events};
use naviterm::tui::Tui;
use std::io;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use config::{Config, ConfigError};
use log4rs::append::file::FileAppender;
use log4rs::Config as log4rsConfig;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::{error, info, LevelFilter};
use naviterm::dbus;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use naviterm::music_database::MusicDatabase;

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
    // Try to load database
    match load_from_disk::<MusicDatabase>("database.bin") {
        Ok(loaded_data) => app.database = loaded_data,
        Err(_e) => app.populate_db().await?,
    }
    //return Ok(());
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
            Event::Key(key_event) => handle_key_events(key_event, &mut app, &iface_ref).await?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::Dbus(dbus_event) => handle_dbus_events(dbus_event, &mut app, &iface_ref).await?
        }
    }

    // Exit the user interface.
    tui.exit()?;
    // Save music database if it does not exist
    match save_to_disk(&app.database,"database.bin") {
        Ok(..) => info!("Database saved successfully!\n"),
        Err(_e) => error!("Error saving database!\n")
    }
    Ok(())
}

fn save_to_disk<T: Serialize>(data: &T, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the file exists
    if Path::new(filename).exists() {
        info!("Database already exists in disk\n");
        return Ok(())
    }
    // Serialize the struct into a byte array
    let encoded: Vec<u8> = bincode::serialize(data)?;
    // Write the serialized data to a file
    let mut file = File::create(filename)?;
    file.write_all(&encoded)?;
    Ok(())
}

fn load_from_disk<T: for<'de> Deserialize<'de>>(
    filename: &str,
) -> Result<T, Box<dyn std::error::Error>> {
    // Check if the file exists
    if !Path::new(filename).exists() {
        return Err("File does not exist.".into());
    }

    let mut file = File::open(filename)?;
    let mut encoded = Vec::new();
    file.read_to_end(&mut encoded)?;

    let decoded: T =
        bincode::deserialize(&encoded).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    Ok(decoded)
}
