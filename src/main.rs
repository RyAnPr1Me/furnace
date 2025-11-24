use anyhow::Result;
use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod config;
mod shell;
mod terminal;
mod ui;
mod plugins;
mod session;
mod keybindings;
mod colors;
mod translator;

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

    // Initialize logging
    let log_level = if args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

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

    // Create and run terminal
    let mut terminal = Terminal::new(config)?;
    terminal.run().await?;

    Ok(())
}
