//! `mosaico settings`: opens the settings app.
//!
//! The settings app is a separate executable installed beside mosaico.
//! It is started in the background, so the command returns at once and
//! closing the terminal leaves the window open.

use super::context::{Context, Exit, complain, say};

/// Starts the settings app beside the running executable.
pub fn execute(cx: &mut Context) -> Exit {
    let exe = match (cx.current_exe)() {
        Ok(exe) => exe,
        Err(e) => {
            complain!(cx, "Error: cannot determine current exe path: {e}");
            return Exit::Failure;
        }
    };
    let settings = exe.with_file_name(cx.installer.settings_name());

    if cx.files.is_file(&settings) == Some(false) {
        complain!(cx, "Error: the settings app is not installed.");
        complain!(cx, "Expected it at {}.", settings.display());
        complain!(cx, "Run 'mosaico update --force' to install it.");
        return Exit::Failure;
    }

    match cx.processes.launch(&settings) {
        Ok(_) => {
            say!(cx, "Opening settings...");
            Exit::Success
        }
        Err(e) => {
            complain!(cx, "Error: {e}");
            Exit::Failure
        }
    }
}

#[cfg(test)]
#[path = "tests/settings_tests.rs"]
mod tests;
