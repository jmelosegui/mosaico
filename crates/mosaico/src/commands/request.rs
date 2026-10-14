//! Sending a command to the daemon from a CLI command.

use mosaico_core::ipc::{
    Command, DAEMON_VERSION, DaemonStatus, ErrorCode, IpcClient, IpcError, Response, ResponseStatus,
};

use crate::platform;

/// Sends `command` to the running daemon and returns its successful
/// reply. Prints why and exits with status 1 otherwise.
pub fn send_or_exit(command: &Command) -> Response {
    let ipc = platform::ipc();
    if !ipc.is_daemon_running() {
        eprintln!("Mosaico is not running.");
        std::process::exit(1);
    }
    match ipc.send(command) {
        Ok(response) if response.status == ResponseStatus::Ok => response,
        Ok(response) => {
            eprintln!("Error: {}", describe_refusal(&response));
            std::process::exit(1);
        }
        Err(IpcError::NotRunning) => {
            eprintln!("Mosaico is not running.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to reach the daemon: {e}");
            std::process::exit(1);
        }
    }
}

/// Explains a failed reply, with what to do about a version mismatch.
pub fn describe_refusal(response: &Response) -> String {
    if response.code == Some(ErrorCode::VersionMismatch) {
        let daemon = response.data::<DaemonStatus>().map_or_else(
            || "another version".to_string(),
            |s| format!("version {}", s.version),
        );
        return format!(
            "the running daemon is {daemon} and this CLI is version {DAEMON_VERSION}. \
             Restart it with `mosaico stop` and `mosaico start`."
        );
    }
    response
        .message
        .clone()
        .unwrap_or_else(|| "unknown error".into())
}

/// Prints the message of a successful reply, if it has one.
pub fn print_message(response: Response) {
    if let Some(message) = response.message {
        println!("{message}");
    }
}
