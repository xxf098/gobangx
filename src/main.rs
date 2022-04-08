mod app;
mod cli;
mod clipboard;
mod components;
mod config;
mod database;
mod event;
mod ui;
mod version;
mod sql;

#[macro_use]
mod log;

use crate::app::App;
use crate::event::{Event, Key};
use anyhow::Result;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

// TODO: SQL meta, foreign table
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let value = crate::cli::parse();
    let connection = value.url.as_ref().map(|u| config::Connection::new(u).ok()).flatten();
    let config = config::Config::new(&value)?;

    setup_terminal()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut events = event::Events::new(250);
    let mut app = App::new(&config, events.sender());
    app.update_databases_internal(connection.as_ref()).await?;

    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            if let Err(err) = app.draw(f) {
                outln!(config#Error, "error: {}", err.to_string());
                std::process::exit(1);
            }
        })?;
        let next_event = events.next().await.unwrap();
        match next_event {
            Event::Input(key) => match app.event(key).await {
                Ok(state) => {
                    if !state.is_consumed()
                        && (key == app.config.key_config.quit || key == app.config.key_config.exit)
                    {
                        break;
                    }
                }
                Err(err) => app.error.set(err.to_string())?,
            },
            Event::Tick => app.clear_keys(),
            _  => match app.action_event(next_event).await {
                Ok(_) => {},
                Err(err) => app.error.set(err.to_string())?,
            },
        }
    }

    shutdown_terminal();
    terminal.show_cursor()?;

    Ok(())
}

fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    Ok(())
}

fn shutdown_terminal() {
    let leave_screen = io::stdout().execute(LeaveAlternateScreen).map(|_f| ());

    if let Err(e) = leave_screen {
        eprintln!("leave_screen failed:\n{}", e);
    }

    let leave_raw_mode = disable_raw_mode();

    if let Err(e) = leave_raw_mode {
        eprintln!("leave_raw_mode failed:\n{}", e);
    }
}
