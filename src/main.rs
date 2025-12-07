use anyhow::{Context, Result};
use clap::Parser;
use std::io::IsTerminal;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod colors;
mod config;
mod keybindings;
mod progress_bar;
mod session;
mod shell;
mod terminal;
mod ui;

use config::Config;
use terminal::Terminal;

/// Furnace - An extremely advanced, high-performance terminal emulator for Windows
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Shell command to execute
    #[arg(short, long)]
    shell: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging to stderr instead of stdout
    // This prevents log messages from appearing in the terminal UI
    let log_level = if args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_writer(std::io::stderr) // Write to stderr, not stdout
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set global default subscriber")?;

    // Load configuration
    let config = if let Some(config_path) = args.config {
        Config::load_from_file(&config_path)?
    } else {
        Config::load_default()?
    };

    // Override shell if specified
    let mut config = config;
    if let Some(shell) = args.shell {
        config.shell.default_shell = shell;
    }

    // Check if stdout is a TTY
    if !std::io::stdout().is_terminal() {
        eprintln!("Error: Furnace must be run in an interactive terminal.");
        eprintln!("It cannot be run with redirected output or in non-TTY environments.");
        eprintln!("\nUsage: Run 'furnace' directly in a terminal emulator.");
        std::process::exit(1);
    }

    // Create and run terminal
    let mut terminal = Terminal::new(config)?;
    // Run terminal with better error context
    if let Err(e) = terminal.run().await {
        // Ensure terminal is cleaned up before showing error
        eprintln!("\nFurnace encountered an error: {}", e);
        eprintln!("\nIf the terminal display is corrupted, try running:");
        eprintln!("  reset");
        return Err(e);
    }

    Ok(())
}
