use std::io;
use tui::{
    backend::CrosstermBackend,
    terminal::Terminal,
};

use almagro::{
    app::App,
    event::EventHandler,
    tui::Tui,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize terminal
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(terminal);
    let handler = EventHandler::new();
    let mut app = App::new()?;

    tui.init()?;
    while app.is_running {
        tui.draw(&mut app)?;

        if let Some(e) = handler.next()? {
            app.handle_keys(e);
        }
    }

    tui.exit()?;
    Ok(())
}
