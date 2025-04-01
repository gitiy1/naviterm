use config::{Config, ConfigError};
use core::panic;
use log::{debug, error, info, warn, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Config as log4rsConfig;
use naviterm::app::{App, AppConnectionMode, AppResult};
use naviterm::dbus;
use naviterm::event::{Event, EventHandler};
use naviterm::handler::{handle_dbus_events, handle_key_events};
use naviterm::music_database::MusicDatabase;
use naviterm::tui::Tui;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use serde::{Deserialize, Serialize};
use std::fs::{copy, remove_file, File};
use std::io;
use std::io::{Read, Write};
use std::path::Path;
use std::process::exit;

#[tokio::main]
async fn main() -> AppResult<()> {
    //Load config
    let home_dir = dirs::home_dir().unwrap();
    let xdg_conf = home_dir.to_string_lossy().to_string() + "/.config/naviterm/";
    let config_file = xdg_conf.clone() + "config.ini";
    let database_file = xdg_conf.clone() + "database.bin";
    let settings = match Config::builder()
        .add_source(config::File::with_name(config_file.as_str()))
        .add_source(config::Environment::with_prefix("APP"))
        .build() {
        Ok(conf) => conf,
        Err(_) => {
            println!("You need to have a config file: HOME/.config/naviterm/config.ini");
            exit(1);
        }
    };
    

    let debug_level: Result<String, ConfigError> = settings.get("debug");
    let level: LevelFilter = match debug_level {
        Ok(level) => match level.as_str() {
            "DEBUG" => LevelFilter::Debug,
            "WARN" => LevelFilter::Warn,
            "ERROR" => LevelFilter::Error,
            "INFO" => LevelFilter::Info,
            _ => LevelFilter::Info,
        },
        Err(_) => LevelFilter::Info,
    };

    // Set the logging path
    let file_path = "/tmp/naviterm.log";

    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new(
            "{h({d(%+)(utc)} [{f}:{L}] {l:<6} {m})}{n}",
        )))
        .build(file_path)
        .unwrap();

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = log4rsConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(level))
        .unwrap();
    let _handle = log4rs::init_config(config);
    
    info!("Starting navidrome...");
    let app_mode: Result<String, ConfigError> = settings.get("mode");
    let mode: AppConnectionMode = match app_mode {
        Ok(mode) => match mode.as_str() {
            "OFFLINE" => AppConnectionMode::Offline,
            _ => AppConnectionMode::Online,
        },
        Err(_) => AppConnectionMode::Online,
    };
    debug!("App connection mode: {:?}", mode);
    
    let replay_gain_mode: Result<String, ConfigError> = settings.get("replay_gain");
    let replay_mode = replay_gain_mode.unwrap_or_else(|_| {
        warn!("No replay gain mode configured, setting to track");
        String::from("track")
    });
    debug!("Replay gain mode: {:?}", replay_mode);

    // Create an application.
    let mut app = App::new();
    app.mode = mode;
    app.app_flags.running = true;
    app.set_config(settings)?;
    app.renew_credentials()?;
    if app.mode != AppConnectionMode::Offline {
        match app.test_connection().await {
            Ok(_) => info!("Connected to server successfully!"),
            Err(_) => {
                app.mode = AppConnectionMode::Offline;
                info!("Could not connect to server, starting offline!")
            }
        }
    }

    // Try to load database
    let loaded = match load_from_disk::<MusicDatabase>(database_file.as_str()) {
        Ok(loaded_data) => {
            app.database = loaded_data;
            info!("Loaded database from file!");
            true
        }
        Err(e) => {
            error! {"Error loading database file: {}", e};
            app.database.populate_default_album();
            false
        }
    };

    // Refresh database
    if app.mode == AppConnectionMode::Online {
        // If we have not loaded a database, fetch it whole
        app.populate_db(!loaded)?;
    } else if !loaded && app.mode == AppConnectionMode::Offline {
        error!("Cannot start offline if no database is present.");
        println!("Cannot start offline if no database is present.");
        exit(1)
    }

    // Initialize ipc stream
    match app.initialize_player_stream() {
        Ok(_) => {
            info!("Initialized ipc stream!")
        }
        Err(e) => {
            error!("Could not initialize ipc stream: {}", e);
            panic!("Could not initialize ipc stream: {}", e);
        }
    }

    if replay_mode == "auto" {
        app.app_flags.replay_gain_auto = true;
        app.set_replay_gain("album")?;
    } else if replay_mode == "track" || replay_mode == "album" {
        app.set_replay_gain(replay_mode.as_str())?;
    } else {
        warn!("Unsupported replay mode: {}. Setting to track", replay_mode);
        app.set_replay_gain("track")?;
    }

    app.poll_player_events().await?;
    info!("Started polling mpv events!");
    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    app.set_event_handler(events.sender.clone()).await?;
    let dbus_connection = dbus::set_up_mpris(events.sender.clone()).await?;
    let mut tui = Tui::new(terminal, events);
    tui.init()?;
    info!("TUI initialized!");

    let iface_ref = dbus_connection
        .object_server()
        .interface::<_, dbus::MediaPlayer2Player>("/org/mpris/MediaPlayer2")
        .await?;
    info!("Initialized dbus interface!");

    // Start the main loop.
    while app.app_flags.running {
        // Render the user interface.
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => {
                if let Err(e) = app.tick() { error!("Unmanaged error while processing the tick event: {}", e) }
            },
            Event::Key(key_event) => {
                if let Err(e) = handle_key_events(key_event, &mut app, &iface_ref).await { error!("Unmanaged error while processing the key event: {}", e) }
                if let Err(e) = tui.draw(&mut app) { error!("Unmanaged error while drawing the UI: {}", e) }
            },
            Event::Resize(_, _) => {}
            Event::Dbus(dbus_event) => {
                if let Err(e) = handle_dbus_events(dbus_event, &mut app, &iface_ref).await { error!("Unmanaged error while processing the dbus event: {}", e) }
            }
            Event::Draw(force_draw) => {
                if app.app_focused || force_draw {
                    if let Err(e) = tui.draw(&mut app) { error!("Unmanaged error while drawing the UI: {}", e) }
                }
            }
            Event::FocusGained => { 
                if !app.app_config.draw_while_unfocused {
                    debug!("Application gained focus, resuming drawing");
                    app.app_focused = true;
                }
            }
            Event::FocusLost => {
                if !app.app_config.draw_while_unfocused {
                    debug!("Application lost focus, will not draw");
                    app.app_focused = false;
                    if let Err(e) = tui.draw(&mut app) { error!("Unmanaged error while drawing the UI: {}", e) }
                }
            }
        }
    }

    // Exit the user interface.
    if let Err(e) = tui.exit() { error!("Unmanaged error while exiting tui: {}", e); }
    // Save music database if it does not exist
    match save_to_disk(&app.database, database_file.as_str()) {
        Ok(..) => info!("Database saved successfully!"),
        Err(e) => error!("Error saving database: {}", e.to_string()),
    }
    Ok(())
}

fn save_to_disk<T: Serialize>(data: &T, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the file exists
    if Path::new(filename).exists() {
        debug!("Database already exists in disk, backing up before saving");
        copy(filename, "/tmp/database_back.bin")?;
        remove_file(filename)?;
    }
    // Serialize the struct into a byte array
    let encoded: Vec<u8> = bincode::serialize(data)?;
    // Write the serialized data to a file
    let mut file = File::create(filename)?;
    file.write_all(&encoded)?;
    // All went well, delete backup
    remove_file("/tmp/database_back.bin")?;
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
