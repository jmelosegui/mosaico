mod commands;
mod platform;

use clap::{Parser, Subcommand};

use mosaico_core::Action;
use mosaico_core::action::Direction;

use commands::context::{Context, Exit};

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
    /// Show the animated mosaico banner
    Banner,
    /// Debugging and inspection tools
    Debug {
        #[command(subcommand)]
        command: DebugCommands,
    },
    /// Pause all mosaico hotkeys until unpaused
    Pause,
    /// Resume all mosaico hotkeys
    Unpause,
    /// Update mosaico to the latest release
    Update {
        /// Reinstall even if already on the latest version
        #[arg(long)]
        force: bool,
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
    /// Cycle to the next layout on the focused workspace
    CycleLayout,
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
    /// Dump the daemon's internal workspace state
    State,
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

    // Refuse to run on a mistyped instance rather than fall back to the
    // user's own daemon and config.
    if let Err(e) = mosaico_core::Instance::current() {
        eprintln!("Error: {e}");
        std::process::exit(2);
    }

    // Before anything looks at the desktop, so the daemon and every
    // command see windows and monitors in the same coordinates.
    platform::init();

    let exit = match cli.command {
        Commands::Daemon => {
            commands::daemon::execute(platform::daemon_startup(), &mut std::io::stderr())
        }
        Commands::Debug {
            command: DebugCommands::Events,
        } => {
            commands::debug::events::execute(&platform::event_source(), &platform::interrupts());
            Exit::Success
        }
        command => platform::run(|cx| run(cx, command)),
    };
    std::process::exit(exit.code());
}

/// Runs a command that goes through the context.
fn run(cx: &mut Context, command: Commands) -> Exit {
    match command {
        Commands::Init => commands::init::execute(cx),
        Commands::Start => commands::start::execute(cx),
        Commands::Stop => commands::stop::execute(cx),
        Commands::Status => commands::status::execute(cx),
        Commands::Doctor => commands::doctor::execute(cx),
        Commands::Autostart { action } => match action {
            AutostartCommands::Enable => commands::autostart::enable(cx),
            AutostartCommands::Disable => commands::autostart::disable(cx),
            AutostartCommands::Status => commands::autostart::status(cx),
        },
        Commands::Pause => commands::pause::pause(cx),
        Commands::Unpause => commands::pause::unpause(cx),
        Commands::Banner => commands::banner::execute(cx),
        Commands::Update { force } => commands::update::execute(cx, force),
        Commands::Action { action } => {
            let action = match action {
                ActionCommands::Focus { direction: d } => Action::Focus(direction(d)),
                ActionCommands::Move { direction: d } => Action::Move(direction(d)),
                ActionCommands::Retile => Action::Retile,
                ActionCommands::ToggleMonocle => Action::ToggleMonocle,
                ActionCommands::CloseFocused => Action::CloseFocused,
                ActionCommands::CycleLayout => Action::CycleLayout,
                ActionCommands::GoToWorkspace { n } => Action::GoToWorkspace(n),
                ActionCommands::SendToWorkspace { n } => Action::SendToWorkspace(n),
            };
            commands::action::execute(cx, action)
        }
        Commands::Debug {
            command: DebugCommands::List,
        } => commands::debug::list::execute(cx),
        Commands::Debug {
            command: DebugCommands::Move(args),
        } => commands::debug::move_window::execute(cx, &args),
        Commands::Debug {
            command: DebugCommands::State,
        } => commands::debug::state::execute(cx),
        // Handled in `main` without a context.
        Commands::Daemon
        | Commands::Debug {
            command: DebugCommands::Events,
        } => Exit::Success,
    }
}
