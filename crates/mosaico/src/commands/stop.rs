use std::time::{Duration, Instant};

use mosaico_core::ipc::ResponseStatus;

/// How long to wait for the daemon process to actually exit.
///
/// The daemon acknowledges the stop command from its IPC thread and
/// then unwinds, which means closing the event hooks, restoring windows
/// and dropping the single instance mutex. Generous enough to cover a
/// loaded machine, short enough that a wedged daemon does not hold the
/// CLI indefinitely.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

/// How often to check whether the process has gone.
const POLL_INTERVAL: Duration = Duration::from_millis(25);

pub fn execute() {
    // Read the pid before anything else. The daemon owns the pid file
    // while it is alive, and the wait below needs something to watch.
    let pid = mosaico_core::pid::read_pid_file().ok().flatten();

    // Try graceful shutdown via IPC first.
    if mosaico_windows::ipc::is_daemon_running() {
        let command = mosaico_core::Command::Stop;
        match mosaico_windows::ipc::send_command(&command) {
            Ok(response) if response.status == ResponseStatus::Ok => {
                let message = response.message.unwrap_or_default();

                // The acknowledgement only means the daemon accepted the
                // command. It holds the single instance mutex until the
                // process exits, so returning now would let a start that
                // follows be rejected with nothing left running.
                let exited = match pid {
                    Some(pid) => wait_until_gone(
                        || mosaico_windows::process::is_process_alive(pid),
                        SHUTDOWN_TIMEOUT,
                        POLL_INTERVAL,
                    ),
                    // No pid file to watch. Nothing to wait on, so fall
                    // back to the old behaviour rather than blocking.
                    None => true,
                };

                let _ = mosaico_core::pid::remove_pid_file();

                if exited {
                    println!("Mosaico stopped. {message}");
                } else {
                    println!("Mosaico stopped. {message}");
                    eprintln!(
                        "Warning: the daemon acknowledged the stop but was still running after \
                         {}s. Starting again may fail until it exits.",
                        SHUTDOWN_TIMEOUT.as_secs()
                    );
                }
                return;
            }
            Ok(response) => {
                eprintln!(
                    "Error: {}",
                    response.message.unwrap_or("unknown error".into())
                );
                return;
            }
            Err(e) => eprintln!("IPC failed: {e}"),
        }
    }

    // Fallback: the IPC pipe is gone but the process may still be
    // alive (e.g. the IPC thread crashed). Check the PID file.
    match mosaico_core::pid::read_pid_file() {
        Ok(Some(pid)) if mosaico_windows::process::is_process_alive(pid) => {
            if mosaico_windows::process::kill_process(pid) {
                // A killed process releases its handles as it is torn
                // down, which is not instant either.
                wait_until_gone(
                    || mosaico_windows::process::is_process_alive(pid),
                    SHUTDOWN_TIMEOUT,
                    POLL_INTERVAL,
                );
                let _ = mosaico_core::pid::remove_pid_file();
                println!("Mosaico stopped (killed PID {pid}).");
            } else {
                eprintln!("Failed to kill process {pid}.");
                std::process::exit(1);
            }
        }
        _ => {
            println!("Mosaico is not running.");
        }
    }
}

/// Polls `alive` until it reports false or `timeout` elapses.
///
/// Returns whether it went away in time. Takes the predicate as a
/// closure so the timing behaviour can be tested without a process.
fn wait_until_gone(mut alive: impl FnMut() -> bool, timeout: Duration, poll: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if !alive() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(poll);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn returns_immediately_when_the_process_is_already_gone() {
        // Arrange
        let calls = Cell::new(0);

        // Act
        let gone = wait_until_gone(
            || {
                calls.set(calls.get() + 1);
                false
            },
            Duration::from_secs(10),
            Duration::from_millis(1),
        );

        // Assert -- one check, no sleeping.
        assert!(gone);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn waits_for_a_process_that_is_still_unwinding() {
        // Arrange -- alive for the first few polls, which is the window
        // between the daemon acknowledging the stop and releasing the
        // single instance mutex.
        let calls = Cell::new(0);

        // Act
        let gone = wait_until_gone(
            || {
                calls.set(calls.get() + 1);
                calls.get() < 4
            },
            Duration::from_secs(10),
            Duration::from_millis(1),
        );

        // Assert
        assert!(gone);
        assert_eq!(calls.get(), 4);
    }

    #[test]
    fn gives_up_on_a_daemon_that_never_exits() {
        // Arrange
        let start = Instant::now();

        // Act
        let gone = wait_until_gone(
            || true,
            Duration::from_millis(60),
            Duration::from_millis(10),
        );

        // Assert -- reports failure rather than blocking the CLI, and
        // does not return before the timeout either.
        assert!(!gone);
        assert!(start.elapsed() >= Duration::from_millis(60));
    }
}
