use naviterm::app::{App, AppResult};
use naviterm::event::{Event, EventHandler};
use naviterm::handler::handle_key_events;
use naviterm::tui::Tui;
use std::io;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use config::Config;

#[tokio::main]
async fn main() -> AppResult<()> {
    //Load config
    let home_dir = dirs::home_dir().unwrap();
    let mut xdg_conf = home_dir.clone();
    xdg_conf.push(".config/naviterm/config.ini");
    let settings = Config::builder()
        // Add in `./Settings.toml`
        .add_source(config::File::with_name(xdg_conf.to_str().unwrap()))
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();
    // Create an application.
    let mut app = App::new();
    app.set_config(settings)?;
    app.renew_credentials()?;
    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

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
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
