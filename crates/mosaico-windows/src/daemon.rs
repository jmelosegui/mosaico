use mosaico_core::WindowResult;
use mosaico_core::ipc::{Command, Response};
use mosaico_core::pid;

use crate::ipc::PipeServer;

/// Runs the Mosaico daemon.
///
/// This is the main loop that:
/// 1. Writes a PID file for process tracking
/// 2. Creates a named pipe server
/// 3. Waits for CLI commands
/// 4. Processes each command
/// 5. Sends a response back
/// 6. Repeats until a Stop command is received
/// 7. Cleans up the PID file on exit
pub fn run() -> WindowResult<()> {
    pid::write_pid_file()?;
    println!("Mosaico daemon started. Listening for commands...");

    let result = daemon_loop();

    // Clean up PID file regardless of how the loop ended.
    let _ = pid::remove_pid_file();

    result
}

/// The inner daemon loop, separated so cleanup always runs in `run()`.
fn daemon_loop() -> WindowResult<()> {
    loop {
        // Create a new pipe instance for each connection.
        // Named pipes on Windows are single-use per connection — after a
        // client disconnects, we need a fresh pipe instance.
        let server = PipeServer::create()?;

        let command = match server.accept_command() {
            Ok(cmd) => cmd,
            Err(e) => {
                eprintln!("Error reading command: {e}");
                continue;
            }
        };

        println!("Received command: {command:?}");

        match command {
            Command::Stop => {
                let response = Response::ok_with_message("Daemon stopping");
                let _ = server.send_response(&response);
                println!("Stop command received. Shutting down.");
                return Ok(());
            }
            Command::Status => {
                let response = Response::ok_with_message("Daemon is running");
                let _ = server.send_response(&response);
            }
        }
    }
}
