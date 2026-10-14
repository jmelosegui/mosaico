use std::io::Write;

use mosaico_core::lifecycle::{self, Startup};

use super::context::Exit;

/// Runs the daemon in this process until it is stopped.
///
/// Why it could not run goes to `err`. Nobody sees it when `start`
/// launched the daemon, but a user who runs `mosaico daemon` in a
/// terminal does.
pub fn execute(startup: Startup<'_>, err: &mut dyn Write) -> Exit {
    match lifecycle::run(startup) {
        Ok(()) => Exit::Success,
        Err(e) => {
            let _ = writeln!(err, "Daemon error: {e}");
            Exit::Failure
        }
    }
}

#[cfg(test)]
#[path = "tests/daemon_tests.rs"]
mod tests;
