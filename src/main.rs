use clap::{Parser, Subcommand};
use anyhow::Result;

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
    Up,
    /// Stop the devbox without removing it
    Down,
    /// Destroy the devbox and all associated resources
    Destroy,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Up => up(),
        Commands::Down => down(),
        Commands::Destroy => destroy(),
    }
}

fn up() -> Result<()> {
    todo!()
}

fn down() -> Result<()> {
    todo!()
}

fn destroy() -> Result<()> {
    todo!()
}
