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
    /// Print what changes on the desktop as it happens
    Events {
        /// Print each event as one JSON object per line
        #[arg(long)]
        json: bool,
        /// Only these events, such as focus_changed or workspace_changed
        kinds: Vec<String>,
    },
    /// Run and manage Lua scripts that drive mosaico
    Script {
        #[command(subcommand)]
        action: ScriptCommands,
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
enum ScriptCommands {
    /// Run a script, by its name in the scripts folder or by its path
    Run {
        /// The script's name, or the path of a .lua file
        script: String,
        /// Arguments the script reads from mosaico.args, as key=value
        args: Vec<String>,
        /// Stop at the first error in a handler or timer
        #[arg(long)]
        fail_fast: bool,
        /// Run it detached, without a console, writing to its log
        #[arg(long)]
        background: bool,
        /// Write what the script prints to this file
        #[arg(long, hide = true)]
        log: Option<std::path::PathBuf>,
    },
    /// Start a script when you sign in, or stop doing so
    Autostart {
        #[command(subcommand)]
        action: ScriptAutostartCommands,
    },
    /// Create a script from a template and open it
    New {
        /// The script's name, such as meeting-mode
        name: String,
    },
    /// Open a script
    Edit {
        /// The script's name
        name: String,
    },
    /// List the scripts
    List,
    /// Copy a script into the scripts folder
    Add {
        /// The .lua file to copy
        path: std::path::PathBuf,
    },
    /// Open the scripts folder
    Open,
}

#[derive(Subcommand)]
enum ScriptAutostartCommands {
    /// Start the script in the background whenever you sign in
    Enable {
        /// The script's name
        name: String,
    },
    /// Stop starting the script when you sign in
    Disable {
        /// The script's name
        name: String,
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
        Commands::Autostart { action } => match action {
            AutostartCommands::Enable => commands::autostart::enable(cx),
            AutostartCommands::Disable => commands::autostart::disable(cx),
            AutostartCommands::Status => commands::autostart::status(cx),
        },
        Commands::Pause => commands::pause::pause(cx),
        Commands::Unpause => commands::pause::unpause(cx),
        Commands::Events { json, kinds } => commands::events::execute(cx, json, &kinds),
        Commands::Script { action } => match action {
            ScriptCommands::Run {
                script,
                args,
                fail_fast,
                background,
                log,
            } => {
                let run = commands::script::run::Run {
                    target: &script,
                    args: &args,
                    fail_fast,
                    background,
                    log: log.as_deref(),
                };
                commands::script::run::execute(cx, &run)
            }
            ScriptCommands::Autostart { action } => match action {
                ScriptAutostartCommands::Enable { name } => {
                    commands::script::autostart::enable(cx, &name)
                }
                ScriptAutostartCommands::Disable { name } => {
                    commands::script::autostart::disable(cx, &name)
                }
            },
            ScriptCommands::New { name } => commands::script::files::new(cx, &name),
            ScriptCommands::Edit { name } => commands::script::files::edit(cx, &name),
            ScriptCommands::List => commands::script::files::list(cx),
            ScriptCommands::Add { path } => commands::script::files::add(cx, &path),
            ScriptCommands::Open => commands::script::files::open_folder(cx),
        },
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
        | Commands::Debug {
            command: DebugCommands::Events,
        } => Exit::Success,
    }
}
