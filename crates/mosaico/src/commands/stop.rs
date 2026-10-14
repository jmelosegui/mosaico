use std::time::Duration;

use mosaico_core::Command;
use mosaico_core::ipc::{IpcError, ResponseStatus};

use super::context::{Clock, Context, Exit, complain, say};
use super::request;

/// How long to wait for the daemon process to actually exit.
///
/// The daemon acknowledges the stop command from its IPC thread and
/// then unwinds, which means closing the event hooks, restoring windows
/// and dropping the single instance mutex. Generous enough to cover a
/// loaded machine, short enough that a wedged daemon does not hold the
/// CLI indefinitely.
pub(super) const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

/// How often to check whether the process has gone.
const POLL_INTERVAL: Duration = Duration::from_millis(25);

pub fn execute(cx: &mut Context) -> Exit {
    // Read the pid before anything else. The daemon owns the pid file
    // while it is alive, and the wait below needs something to watch.
    let pid = cx.pid_file.read().ok().flatten();

    // Try graceful shutdown via IPC first. There is no liveness check
    // before it: a daemon busy with another client would look missing.
    match cx.daemon.send(&Command::Stop) {
        Ok(response) if response.status == ResponseStatus::Ok => {
            let message = response.message.unwrap_or_default();

            // The acknowledgement only means the daemon accepted the
            // command. It holds the single instance mutex until the
            // process exits, so returning now would let a start that
            // follows be rejected with nothing left running.
            let exited = match pid {
                Some(pid) => wait_until_gone(
                    || cx.processes.is_alive(pid),
                    cx.clock,
                    SHUTDOWN_TIMEOUT,
                    POLL_INTERVAL,
                ),
                // No pid file to watch, so there is nothing to wait on.
                None => true,
            };

            let _ = cx.pid_file.remove();
            say!(cx, "Mosaico stopped. {message}");
            if !exited {
                complain!(
                    cx,
                    "Warning: the daemon acknowledged the stop but was still running after \
                     {}s. Starting again may fail until it exits.",
                    SHUTDOWN_TIMEOUT.as_secs()
                );
            }
            return Exit::Success;
        }
        Ok(response) => {
            complain!(cx, "Error: {}", request::describe_refusal(&response));
            return Exit::Failure;
        }
        Err(IpcError::NotRunning) => {}
        Err(e) => complain!(cx, "IPC failed: {e}"),
    }

    // Fallback: the IPC pipe is gone but the process may still be
    // alive (e.g. the IPC thread crashed). Check the PID file.
    match cx.pid_file.read() {
        Ok(Some(pid)) if cx.processes.is_alive(pid) => {
            if !cx.processes.kill(pid) {
                complain!(cx, "Failed to kill process {pid}.");
                return Exit::Failure;
            }
            // A killed process releases its handles as it is torn
            // down, which is not instant either.
            wait_until_gone(
                || cx.processes.is_alive(pid),
                cx.clock,
                SHUTDOWN_TIMEOUT,
                POLL_INTERVAL,
            );
            let _ = cx.pid_file.remove();
            say!(cx, "Mosaico stopped (killed PID {pid}).");
        }
        _ => say!(cx, "Mosaico is not running."),
    }
    Exit::Success
}

/// Polls `alive` until it reports false or `timeout` elapses.
///
/// Returns whether it went away in time.
fn wait_until_gone(
    mut alive: impl FnMut() -> bool,
    clock: &dyn Clock,
    timeout: Duration,
    poll: Duration,
) -> bool {
    let deadline = clock.now() + timeout;
    loop {
        if !alive() {
            return true;
        }
        if clock.now() >= deadline {
            return false;
        }
        clock.sleep(poll);
    }
}

#[cfg(test)]
#[path = "tests/stop_tests.rs"]
mod tests;
