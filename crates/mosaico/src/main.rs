mod action_commands;
mod commands;
mod platform;

use clap::{Parser, Subcommand};

use action_commands::ActionCommands;
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
    Start {
        /// Print nothing and skip the update check, as when starting at login
        #[arg(long)]
        quiet: bool,
    },
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
    /// Print the state bars are shown, once
    Query(commands::query::QueryArgs),
    /// Follow the state bars are shown, a line for each change
    Subscribe(commands::subscribe::SubscribeArgs),
    /// Run an action as a click on a bar runs it
    Run(commands::run::RunArgs),
    /// Set up a bar another program draws
    Bar {
        #[command(subcommand)]
        command: BarCommands,
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
    /// Open the settings window
    Settings,
    /// Update mosaico to the latest release
    Update {
        /// Reinstall even if already on the latest version
        #[arg(long)]
        force: bool,
    },
    /// Run the daemon (internal — not for direct use)
    #[command(hide = true)]
    Daemon,
    /// Run a step of the installer package (internal — not for direct use)
    #[command(hide = true)]
    InstallerHook {
        #[arg(value_enum)]
        hook: commands::installer_hook::Hook,
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
enum BarCommands {
    /// Print the widgets a bar needs to show mosaico, and pick it once it runs
    Setup {
        /// The bar: yasb
        bar: String,
    },
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

fn main() {
    let cli = Cli::parse();

    // Refuse to run on a mistyped instance rather than fall back to the
    // user's own daemon and config, or when there is nowhere to keep the
    // instance's files.
    if let Err(e) = platform::resolve_instance() {
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
            commands::debug::events::execute(
                &platform::event_source(),
                &platform::interrupts(),
                &platform::desktop(),
            );
            Exit::Success
        }
        Commands::Query(args) => platform::bar_api(|io| commands::query::execute(io, args)),
        Commands::Subscribe(args) => platform::bar_api(|io| commands::subscribe::execute(io, args)),
        Commands::Run(args) => platform::bar_api(|io| commands::run::execute(io, args)),
        command => platform::run(|cx| run(cx, command)),
    };
    std::process::exit(exit.code());
}

/// Runs a command that goes through the context.
fn run(cx: &mut Context, command: Commands) -> Exit {
    match command {
        Commands::Init => commands::init::execute(cx),
        Commands::Start { quiet: false } => commands::start::execute(cx),
        Commands::Start { quiet: true } => commands::start::quietly(cx),
        Commands::Stop => commands::stop::execute(cx),
        Commands::Status => commands::status::execute(cx),
        Commands::Doctor => commands::doctor::execute(cx),
        Commands::Bar {
            command: BarCommands::Setup { bar },
        } => commands::bar_setup::execute(cx, &bar),
        Commands::Autostart { action } => match action {
            AutostartCommands::Enable => commands::autostart::enable(cx),
            AutostartCommands::Disable => commands::autostart::disable(cx),
            AutostartCommands::Status => commands::autostart::status(cx),
        },
        Commands::Pause => commands::pause::pause(cx),
        Commands::Unpause => commands::pause::unpause(cx),
        Commands::Banner => commands::banner::execute(cx),
        Commands::Settings => commands::settings::execute(cx),
        Commands::Update { force } => commands::update::execute(cx, force),
        Commands::InstallerHook { hook } => commands::installer_hook::execute(cx, hook),
        Commands::Action { action } => commands::action::execute(cx, action.action()),
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
        | Commands::Query(_)
        | Commands::Subscribe(_)
        | Commands::Run(_)
        | Commands::Debug {
            command: DebugCommands::Events,
        } => Exit::Success,
    }
}
