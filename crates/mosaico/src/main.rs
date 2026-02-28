mod commands;

use clap::{Parser, Subcommand};

use mosaico_core::Action;
use mosaico_core::action::Direction;

#[derive(Parser)]
#[command(
    name = "mosaico",
    version,
    about = "A cross-platform tiling window manager",
    before_help = concat!(
        "\n",
        "  \x1b[94m█▀▄▀█ █▀▀█ █▀▀▀ █▀▀█ ▀█▀ █▀▀▀ █▀▀█\x1b[0m\n",
        "  \x1b[94m█ ▀ █ █  █ ▀▀▀█ █▀▀█  █  █    █  █\x1b[0m\n",
        "  \x1b[94m▀   ▀ ▀▀▀▀ ▀▀▀▀ ▀  ▀ ▀▀▀ ▀▀▀▀ ▀▀▀▀\x1b[0m\n",
        "                              \x1b[90mv", env!("CARGO_PKG_VERSION"), "\x1b[0m",
    ),
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create the default configuration file
    Init,
    /// Start the window manager daemon
    Start,
    /// Stop the window manager daemon
    Stop,
    /// Show whether the daemon is running
    Status,
    /// Check your setup for common issues
    Doctor,
    /// Manage automatic startup when Windows boots
    Autostart {
        #[command(subcommand)]
        action: AutostartCommands,
    },
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
enum DirectionCommands {
    /// Left
    Left,
    /// Right
    Right,
    /// Up
    Up,
    /// Down
    Down,
}

#[derive(Subcommand)]
enum ActionCommands {
    /// Focus a window in the given direction
    Focus {
        #[command(subcommand)]
        direction: DirectionCommands,
    },
    /// Move the focused window in the given direction
    Move {
        #[command(subcommand)]
        direction: DirectionCommands,
    },
    /// Re-apply the current layout
    Retile,
    /// Toggle monocle mode (focused window fills the monitor)
    ToggleMonocle,
    /// Close the currently focused window
    CloseFocused,
    /// Switch to workspace N (1-8) on the focused monitor
    GoToWorkspace {
        /// Workspace number (1-8)
        n: u8,
    },
    /// Send the focused window to workspace N (1-8)
    SendToWorkspace {
        /// Workspace number (1-8)
        n: u8,
    },
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

#[derive(Subcommand)]
enum AutostartCommands {
    /// Enable autostart on Windows boot
    Enable,
    /// Disable autostart on Windows boot
    Disable,
    /// Show current autostart status
    Status,
}

fn direction(d: DirectionCommands) -> Direction {
    match d {
        DirectionCommands::Left => Direction::Left,
        DirectionCommands::Right => Direction::Right,
        DirectionCommands::Up => Direction::Up,
        DirectionCommands::Down => Direction::Down,
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init::execute(),
        Commands::Start => commands::start::execute(),
        Commands::Stop => commands::stop::execute(),
        Commands::Status => commands::status::execute(),
        Commands::Doctor => commands::doctor::execute(),
        Commands::Autostart { action } => match action {
            AutostartCommands::Enable => commands::autostart::enable(),
            AutostartCommands::Disable => commands::autostart::disable(),
            AutostartCommands::Status => commands::autostart::status(),
        },
        Commands::Daemon => commands::daemon::execute(),
        Commands::Action { action } => {
            let action = match action {
                ActionCommands::Focus { direction: d } => Action::Focus(direction(d)),
                ActionCommands::Move { direction: d } => Action::Move(direction(d)),
                ActionCommands::Retile => Action::Retile,
                ActionCommands::ToggleMonocle => Action::ToggleMonocle,
                ActionCommands::CloseFocused => Action::CloseFocused,
                ActionCommands::GoToWorkspace { n } => Action::GoToWorkspace(n),
                ActionCommands::SendToWorkspace { n } => Action::SendToWorkspace(n),
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
