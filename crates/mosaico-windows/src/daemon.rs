use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use mosaico_core::ipc::{Command, Response};
use mosaico_core::{Action, BspLayout, WindowResult, config, pid};

use crate::dpi;
use crate::event_loop;
use crate::ipc::PipeServer;
use crate::tiling::TilingManager;

/// Runs the Mosaico daemon.
///
/// Starts background threads for the Win32 event loop (which also
/// handles global hotkeys) and the IPC listener. The main thread
/// manages the tiling workspace.
pub fn run() -> WindowResult<()> {
    dpi::enable_dpi_awareness();
    pid::write_pid_file()?;
    eprintln!("Mosaico daemon started.");

    let result = daemon_loop();

    let _ = pid::remove_pid_file();

    result
}

/// Internal message type for the main daemon thread.
enum DaemonMsg {
    /// A window event from the event loop.
    Event(mosaico_core::WindowEvent),
    /// A user action from hotkeys or IPC.
    Action(Action),
    /// A CLI command with a callback to send the response.
    Command(Command, ResponseSender),
}

/// Sends a response back to the IPC thread for the connected client.
type ResponseSender = mpsc::Sender<Response>;

/// The inner daemon loop, separated so cleanup always runs in `run()`.
fn daemon_loop() -> WindowResult<()> {
    let config = config::load();
    let keybindings = config::load_keybindings();
    let rules = config::load_rules();
    if let Some(path) = config::config_path() {
        eprintln!("Config: {}", path.display());
    }
    if let Some(path) = config::keybindings_path() {
        eprintln!("Keybindings: {}", path.display());
    }
    if let Some(path) = config::rules_path() {
        eprintln!("Rules: {}", path.display());
    }

    let layout = BspLayout {
        gap: config.layout.gap,
        ratio: config.layout.ratio,
    };

    let (tx, rx) = mpsc::channel::<DaemonMsg>();

    let mut manager = TilingManager::new(layout, rules, config.borders)?;
    eprintln!("Managing {} windows.", manager.window_count());

    // Start the Win32 event loop + hotkeys on its own thread.
    let event_tx = tx.clone();
    let action_tx = tx.clone();
    let (event_channel_tx, event_channel_rx) = mpsc::channel();
    let (action_channel_tx, action_channel_rx) = mpsc::channel();
    let event_loop = event_loop::start(event_channel_tx, action_channel_tx, keybindings)?;

    // Bridge: forward window events into the unified channel.
    let event_bridge = thread::spawn(move || {
        for event in event_channel_rx {
            if event_tx.send(DaemonMsg::Event(event)).is_err() {
                break;
            }
        }
    });

    // Bridge: forward hotkey actions into the unified channel.
    let action_bridge = thread::spawn(move || {
        for action in action_channel_rx {
            if action_tx.send(DaemonMsg::Action(action)).is_err() {
                break;
            }
        }
    });

    // Start the IPC listener on its own thread.
    let ipc_tx = tx.clone();
    let ipc_thread = thread::spawn(move || ipc_loop(ipc_tx));

    // Main processing loop.
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(DaemonMsg::Event(event)) => {
                manager.handle_event(&event);
            }
            Ok(DaemonMsg::Action(action)) => {
                manager.handle_action(&action);
            }
            Ok(DaemonMsg::Command(Command::Stop, reply_tx)) => {
                let response = Response::ok_with_message("Daemon stopping");
                let _ = reply_tx.send(response);
                eprintln!("Stop command received. Shutting down.");
                break;
            }
            Ok(DaemonMsg::Command(Command::Status, reply_tx)) => {
                let msg = format!(
                    "Daemon is running, managing {} windows",
                    manager.window_count()
                );
                let response = Response::ok_with_message(msg);
                let _ = reply_tx.send(response);
            }
            Ok(DaemonMsg::Command(Command::Action { action }, reply_tx)) => {
                manager.handle_action(&action);
                let response = Response::ok();
                let _ = reply_tx.send(response);
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    event_loop.stop();
    drop(tx);
    let _ = event_bridge.join();
    let _ = action_bridge.join();
    let _ = ipc_thread.join();

    Ok(())
}

/// Accepts IPC connections in a loop and forwards commands to the
/// main daemon thread. Runs on a dedicated thread.
fn ipc_loop(tx: mpsc::Sender<DaemonMsg>) {
    loop {
        let server = match PipeServer::create() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to create pipe: {e}");
                return;
            }
        };

        let command = match server.accept_command() {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("Error reading command: {e}");
                continue;
            }
        };

        let (reply_tx, reply_rx) = mpsc::channel();
        let is_stop = matches!(command, Command::Stop);

        if tx.send(DaemonMsg::Command(command, reply_tx)).is_err() {
            return;
        }

        if let Ok(response) = reply_rx.recv() {
            let _ = server.send_response(&response);
        }

        if is_stop {
            return;
        }
    }
}
