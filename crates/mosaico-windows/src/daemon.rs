use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use mosaico_core::ipc::{Command, Response};
use mosaico_core::{WindowResult, pid};

use crate::event_loop;
use crate::ipc::PipeServer;

/// Runs the Mosaico daemon.
///
/// Starts two background threads:
/// - **Event loop**: hooks into Win32 window events and sends them over a channel
/// - **IPC listener**: accepts CLI commands over a named pipe
///
/// The main thread processes both event and command channels until
/// a Stop command is received.
pub fn run() -> WindowResult<()> {
    pid::write_pid_file()?;
    eprintln!("Mosaico daemon started.");

    let result = daemon_loop();

    // Clean up PID file regardless of how the loop ended.
    let _ = pid::remove_pid_file();

    result
}

/// Internal message type for the main daemon thread.
enum DaemonMsg {
    /// A window event from the event loop.
    Event(mosaico_core::WindowEvent),
    /// A CLI command with a callback to send the response.
    Command(Command, ResponseSender),
}

/// Sends a response back to the IPC thread for the connected client.
type ResponseSender = mpsc::Sender<Response>;

/// The inner daemon loop, separated so cleanup always runs in `run()`.
fn daemon_loop() -> WindowResult<()> {
    let (tx, rx) = mpsc::channel::<DaemonMsg>();

    // Start the Win32 event loop on its own thread.
    let event_tx = tx.clone();
    let (event_channel_tx, event_channel_rx) = mpsc::channel();
    let event_loop = event_loop::start(event_channel_tx)?;

    // Bridge: forward window events into the unified daemon channel.
    let event_bridge = thread::spawn(move || {
        for event in event_channel_rx {
            if event_tx.send(DaemonMsg::Event(event)).is_err() {
                break;
            }
        }
    });

    // Start the IPC listener on its own thread.
    let ipc_tx = tx.clone();
    let ipc_thread = thread::spawn(move || ipc_loop(ipc_tx));

    // Main processing loop: handle events and commands.
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(DaemonMsg::Event(event)) => {
                eprintln!("Event: {event:?}");
            }
            Ok(DaemonMsg::Command(Command::Stop, reply_tx)) => {
                let response = Response::ok_with_message("Daemon stopping");
                let _ = reply_tx.send(response);
                eprintln!("Stop command received. Shutting down.");
                break;
            }
            Ok(DaemonMsg::Command(Command::Status, reply_tx)) => {
                let response = Response::ok_with_message("Daemon is running");
                let _ = reply_tx.send(response);
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    // Shut down the event loop and wait for threads to finish.
    event_loop.stop();
    drop(tx);
    let _ = event_bridge.join();
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

        // Create a one-shot channel for the response.
        let (reply_tx, reply_rx) = mpsc::channel();
        let is_stop = matches!(command, Command::Stop);

        if tx.send(DaemonMsg::Command(command, reply_tx)).is_err() {
            return;
        }

        // Wait for the main thread to process the command.
        if let Ok(response) = reply_rx.recv() {
            let _ = server.send_response(&response);
        }

        if is_stop {
            return;
        }
    }
}
