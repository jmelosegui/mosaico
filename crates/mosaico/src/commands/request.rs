//! Sending a command to the daemon from a CLI command.

use mosaico_core::ipc::{
    Command, DAEMON_VERSION, DaemonStatus, ErrorCode, IpcError, Response, ResponseStatus,
};

use super::context::{Context, Exit, complain, say};

/// What to tell the user when the daemon is listening but does not reply.
pub const NOT_RESPONDING: &str = "Mosaico is running but not responding. Run `mosaico stop` to end it, \
     then `mosaico start`.";

/// Sends `command` to the running daemon and returns its successful
/// reply. Otherwise writes why to the error stream and returns the
/// failure to exit with.
pub fn send(cx: &mut Context, command: &Command) -> Result<Response, Exit> {
    // No liveness check first: send reports a daemon that is not
    // running by itself, and a separate check could see a busy daemon
    // as missing.
    match cx.daemon.send(command) {
        Ok(response) if response.status == ResponseStatus::Ok => Ok(response),
        Ok(response) => {
            complain!(cx, "Error: {}", describe_refusal(&response));
            Err(Exit::Failure)
        }
        Err(IpcError::NotRunning) => {
            complain!(cx, "Mosaico is not running.");
            Err(Exit::Failure)
        }
        Err(IpcError::TimedOut) => {
            complain!(cx, "{NOT_RESPONDING}");
            Err(Exit::Failure)
        }
        Err(e) => {
            complain!(cx, "Failed to reach the daemon: {e}");
            Err(Exit::Failure)
        }
    }
}

/// Sends `command` and prints the message of its reply.
pub fn run(cx: &mut Context, command: &Command) -> Exit {
    match send(cx, command) {
        Ok(response) => {
            if let Some(message) = response.message {
                say!(cx, "{message}");
            }
            Exit::Success
        }
        Err(exit) => exit,
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

#[cfg(test)]
#[path = "tests/request_tests.rs"]
mod tests;
