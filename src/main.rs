mod app;
mod browser;
mod diff;
mod ui;

use app::{App, AppMode};
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::process;

/// A terminal-based diff viewer
#[derive(Parser)]
#[command(name = "lazydiff")]
#[command(version)]
#[command(about = "A terminal-based diff viewer", long_about = None)]
struct Cli {
    /// Source file to compare
    source: Option<String>,

    /// Target file to compare against
    target: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // Validate files if provided, before entering TUI mode
    if let Some(source) = &args.source
        && let Err(e) = diff::validate_file(source, "Source")
    {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    if let Some(target) = &args.target
        && let Err(e) = diff::validate_file(target, "Target")
    {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app based on provided arguments
    let app = match (&args.source, &args.target) {
        (Some(source), Some(target)) => {
            // Both files provided - create app normally
            App::new(source.clone(), target.clone())?
        }
        (Some(source), None) => {
            // Source provided, need to select target
            let mut app = App::new_empty(AppMode::SelectingTarget)?;
            app.source_file = source.clone();
            app.status_message = Some(format!("Source: {} - Select target file", source));
            app
        }
        (None, Some(target)) => {
            // Target provided, need to select source
            let mut app = App::new_empty(AppMode::SelectingSource)?;
            app.target_file = target.clone();
            app.status_message = Some(format!("Target: {} - Select source file", target));
            app
        }
        (None, None) => {
            // No files provided - start by selecting source
            App::new_empty(AppMode::SelectingSource)?
        }
    };

    let res = app::run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
        process::exit(1);
    }

    Ok(())
}
