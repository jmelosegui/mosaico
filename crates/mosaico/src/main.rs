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
    /// Start the window manager daemon
    Start,
    /// Stop the window manager daemon
    Stop,
    /// Show whether the daemon is running
    Status,
    /// Debugging and inspection tools
    Debug {
        #[command(subcommand)]
        command: DebugCommands,
    },
    /// Run the daemon (internal — not for direct use)
    #[command(hide = true)]
    Daemon,
}

#[derive(Subcommand)]
enum DebugCommands {
    /// List all visible windows
    List,
    /// Watch window events in real time
    Events,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => commands::start::execute(),
        Commands::Stop => commands::stop::execute(),
        Commands::Status => commands::status::execute(),
        Commands::Daemon => commands::daemon::execute(),
        Commands::Debug { command } => match command {
            DebugCommands::List => commands::debug::list::execute(),
            DebugCommands::Events => commands::debug::events::execute(),
        },
    }
}
