mod commands;

use clap::{Parser, Subcommand};

use mosaico_core::Action;

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
    /// Send an action to the running daemon
    Action {
        #[command(subcommand)]
        action: ActionCommands,
    },
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
enum ActionCommands {
    /// Move focus to the next window
    FocusNext,
    /// Move focus to the previous window
    FocusPrev,
    /// Swap the focused window with the next one
    SwapNext,
    /// Swap the focused window with the previous one
    SwapPrev,
    /// Re-apply the current layout
    Retile,
    /// Move focus to a window on the next monitor
    FocusMonitorNext,
    /// Move focus to a window on the previous monitor
    FocusMonitorPrev,
    /// Move the focused window to the next monitor
    MoveToMonitorNext,
    /// Move the focused window to the previous monitor
    MoveToMonitorPrev,
    /// Toggle monocle mode (focused window fills the monitor)
    ToggleMonocle,
}

#[derive(Subcommand)]
enum DebugCommands {
    /// List all visible windows
    List,
    /// Watch window events in real time
    Events,
    /// Move a window to a specific position and size
    Move(commands::debug::move_window::MoveArgs),
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => commands::start::execute(),
        Commands::Stop => commands::stop::execute(),
        Commands::Status => commands::status::execute(),
        Commands::Daemon => commands::daemon::execute(),
        Commands::Action { action } => {
            let action = match action {
                ActionCommands::FocusNext => Action::FocusNext,
                ActionCommands::FocusPrev => Action::FocusPrev,
                ActionCommands::SwapNext => Action::SwapNext,
                ActionCommands::SwapPrev => Action::SwapPrev,
                ActionCommands::Retile => Action::Retile,
                ActionCommands::FocusMonitorNext => Action::FocusMonitorNext,
                ActionCommands::FocusMonitorPrev => Action::FocusMonitorPrev,
                ActionCommands::MoveToMonitorNext => Action::MoveToMonitorNext,
                ActionCommands::MoveToMonitorPrev => Action::MoveToMonitorPrev,
                ActionCommands::ToggleMonocle => Action::ToggleMonocle,
            };
            commands::action::execute(action);
        }
        Commands::Debug { command } => match command {
            DebugCommands::List => commands::debug::list::execute(),
            DebugCommands::Events => commands::debug::events::execute(),
            DebugCommands::Move(args) => commands::debug::move_window::execute(&args),
        },
    }
}
