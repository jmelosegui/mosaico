use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "mosaico",
    version,
    about = "A tiling window manager for Windows"
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
        Commands::Start => println!("Starting Mosaico..."),
        Commands::Stop => println!("Stopping Mosaico..."),
        Commands::Status => println!("Mosaico status: not running"),
    }
}
