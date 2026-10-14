use mosaico_core::ipc::{
    Command, DAEMON_VERSION, DaemonStatus, IpcError, Response, ResponseStatus,
};

use super::context::{Context, Exit, say};
use super::request;

pub fn execute(cx: &mut Context) -> Exit {
    match cx.daemon.send(&Command::Status) {
        Ok(response) => {
            say!(cx, "{}", describe_running(&response));
            return Exit::Success;
        }
        Err(IpcError::NotRunning) => {}
        Err(e) => {
            say!(cx, "Mosaico is running, but did not answer: {e}");
            return Exit::Success;
        }
    }

    // The pipe is not answering, so check whether a daemon that was
    // killed without a clean shutdown left its PID file behind.
    match cx.pid_file.read() {
        Ok(Some(pid)) if cx.processes.is_alive(pid) => {
            say!(
                cx,
                "Mosaico process exists (PID: {pid}) but is not responding."
            );
        }
        Ok(Some(_)) => {
            let _ = cx.pid_file.remove();
            say!(cx, "Mosaico is not running (cleaned up stale PID file).");
        }
        _ => say!(cx, "Mosaico is not running."),
    }
    Exit::Success
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
    use crate::commands::test_support::{Harness, Lifetime};

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

    #[test]
    fn reports_a_running_daemon_with_its_status() {
        // Arrange
        let mut h = Harness::new("status-running");
        h.world.daemon_up(5);
        h.world
            .reply(Ok(Response::ok().with_data(&status(DAEMON_VERSION))));

        // Act
        let exit = h.run(execute);

        // Assert
        assert_eq!(exit, Exit::Success);
        assert_eq!(
            h.out(),
            format!(
                "Mosaico {DAEMON_VERSION} is running, managing 3 windows.
"
            )
        );
    }

    #[test]
    fn cleans_up_a_stale_pid_file() {
        // Arrange
        let mut h = Harness::new("status-stale");
        h.pid_file.write(99).unwrap();

        // Act
        h.run(execute);

        // Assert
        assert_eq!(
            h.out(),
            "Mosaico is not running (cleaned up stale PID file).
"
        );
        assert_eq!(h.pid_file.read().unwrap(), None);
    }

    #[test]
    fn notices_a_process_that_stopped_answering() {
        // Arrange
        let mut h = Harness::new("status-hung");
        h.world.processes.borrow_mut().insert(99, Lifetime::Forever);
        h.pid_file.write(99).unwrap();

        // Act
        h.run(execute);

        // Assert
        assert_eq!(
            h.out(),
            "Mosaico process exists (PID: 99) but is not responding.
"
        );
        assert_eq!(h.pid_file.read().unwrap(), Some(99), "kept for `stop`");
    }

    #[test]
    fn a_daemon_that_does_not_reply_is_reported_as_not_responding() {
        // Arrange -- the daemon took the request but never answered.
        let mut h = Harness::new("status-no-reply");
        h.world.daemon_up(5);
        h.world.reply(Err(IpcError::TimedOut));

        // Act
        let exit = h.run(execute);

        // Assert
        assert_eq!(exit, Exit::Success);
        assert_eq!(h.out(), format!("{}\n", request::NOT_RESPONDING));
    }

    #[test]
    fn reports_nothing_running() {
        // Arrange
        let mut h = Harness::new("status-none");

        // Act
        h.run(execute);

        // Assert
        assert_eq!(
            h.out(),
            "Mosaico is not running.
"
        );
    }
}
