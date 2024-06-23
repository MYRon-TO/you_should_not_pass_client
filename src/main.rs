use tokio::sync::RwLock;
use you_should_not_pass_client::app::{App, AppResult};
use you_should_not_pass_client::event::{Event, EventHandler};
use you_should_not_pass_client::handler::handle_key_events;
use you_should_not_pass_client::tui::Tui;
use std::io;
use std::sync::Arc;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Create an application.
    // let mut app = App::new();
    let app = Arc::new(RwLock::new(App::new()));

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.read().await.running {
        // Render the user interface.
        tui.draw(app.clone())?;
        // Handle events.
        match tui.events.next().await? {
            // Event::Tick => app.tick(),
            Event::Tick => {},
            Event::Key(key_event) => handle_key_events(key_event, app.clone()).await?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
