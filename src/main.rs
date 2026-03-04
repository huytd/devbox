use clap::{Parser, Subcommand};
use anyhow::Result;

mod config;
mod backend;
mod commands;

#[derive(Parser)]
#[command(name = "devbox")]
#[command(about = "Create and manage isolated dev environments")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create or attach to a devbox environment
    Up {
        #[arg(short = 'p', long = "port", value_name = "HOST:CONTAINER")]
        ports: Vec<String>,
    },
    /// Stop the devbox without removing it
    Down,
    /// Destroy the devbox and all associated resources
    Destroy,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Up { ports } => commands::up(ports),
        Commands::Down => commands::down(),
        Commands::Destroy => commands::destroy(),
    }
}
