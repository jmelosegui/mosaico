use mosaico_core::ipc::{
    Command, DAEMON_VERSION, DaemonStatus, IpcClient, Response, ResponseStatus,
};

use super::request;

pub fn execute() {
    let ipc = crate::platform::ipc();
    if ipc.is_daemon_running() {
        match ipc.send(&Command::Status) {
            Ok(response) => println!("{}", describe_running(&response)),
            Err(e) => println!("Mosaico is running, but did not answer: {e}"),
        }
        return;
    }

    // Pipe is not responding, so check if a stale PID file was left behind
    // by a daemon that was killed without a clean shutdown.
    if let Ok(Some(pid)) = mosaico_core::pid::read_pid_file() {
        if mosaico_windows::process::is_process_alive(pid) {
            println!("Mosaico process exists (PID: {pid}) but is not responding.");
        } else {
            let _ = mosaico_core::pid::remove_pid_file();
            println!("Mosaico is not running (cleaned up stale PID file).");
        }
    } else {
        println!("Mosaico is not running.");
    }
}

/// What to tell the user about a daemon that answered a status request.
fn describe_running(response: &Response) -> String {
    if response.status != ResponseStatus::Ok {
        return format!(
            "Mosaico is running, but {}",
            request::describe_refusal(response)
        );
    }
    match response.data::<DaemonStatus>() {
        Some(status) if status.version != DAEMON_VERSION => format!(
            "Mosaico {} is running, managing {} windows. This CLI is version \
             {DAEMON_VERSION}; restart the daemon with `mosaico stop` and `mosaico start`.",
            status.version, status.windows
        ),
        Some(status) => format!(
            "Mosaico {} is running, managing {} windows.",
            status.version, status.windows
        ),
        // A daemon from before the status payload.
        None => "Mosaico is running.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use mosaico_core::ipc::{ErrorCode, PROTOCOL_VERSION};

    use super::*;

    fn status(version: &str) -> DaemonStatus {
        DaemonStatus {
            version: version.into(),
            protocol: PROTOCOL_VERSION,
            windows: 3,
        }
    }

    #[test]
    fn reports_the_daemon_version_and_window_count() {
        // Arrange
        let response = Response::ok().with_data(&status(DAEMON_VERSION));

        // Act
        let text = describe_running(&response);

        // Assert
        assert_eq!(
            text,
            format!("Mosaico {DAEMON_VERSION} is running, managing 3 windows.")
        );
    }

    #[test]
    fn suggests_a_restart_when_the_daemon_is_another_release() {
        // Arrange
        let response = Response::ok().with_data(&status("0.0.1"));

        // Act
        let text = describe_running(&response);

        // Assert
        assert!(text.contains("Mosaico 0.0.1 is running"), "{text}");
        assert!(text.contains("mosaico stop"), "{text}");
    }

    #[test]
    fn explains_a_protocol_mismatch() {
        // Arrange
        let response =
            Response::error(ErrorCode::VersionMismatch, "mismatch").with_data(&status("0.0.1"));

        // Act
        let text = describe_running(&response);

        // Assert
        assert!(text.contains("version 0.0.1"), "{text}");
        assert!(text.contains("mosaico start"), "{text}");
    }

    #[test]
    fn copes_with_a_daemon_that_sends_no_status() {
        // Arrange -- a daemon from before the status payload.
        let response = Response::ok_with_message("Daemon is running");

        // Act / Assert
        assert_eq!(describe_running(&response), "Mosaico is running.");
    }
}
