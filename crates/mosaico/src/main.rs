mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "mosaico",
    version,
    about = "A cross-platform tiling window manager"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the window manager
    Start,
    /// Stop the window manager
    Stop,
    /// Show current status
    Status,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => commands::start::execute(),
        Commands::Stop => commands::stop::execute(),
        Commands::Status => commands::status::execute(),
    }
}
