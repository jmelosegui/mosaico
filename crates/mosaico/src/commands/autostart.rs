//! Starting mosaico when the user logs in.

use std::path::PathBuf;

use mosaico_core::Error;
use mosaico_core::system::{AutostartStatus, LaunchCommand};

use super::context::{Context, Exit, complain, say};

pub fn enable(cx: &mut Context) -> Exit {
    match register(cx) {
        Ok(()) => {
            say!(cx, "Autostart enabled.");
            Exit::Success
        }
        Err(e) => {
            complain!(cx, "Error: {e}");
            Exit::Failure
        }
    }
}

pub fn disable(cx: &mut Context) -> Exit {
    match cx.autostart.disable() {
        Ok(()) => {
            say!(cx, "Autostart disabled.");
            Exit::Success
        }
        Err(e) => {
            complain!(cx, "Error: {e}");
            Exit::Failure
        }
    }
}

pub fn status(cx: &mut Context) -> Exit {
    match check(cx) {
        Ok(AutostartStatus::Enabled) => say!(cx, "Autostart is currently enabled."),
        Ok(AutostartStatus::Disabled) => say!(cx, "Autostart is currently disabled."),
        Ok(AutostartStatus::TurnedOff) => say!(
            cx,
            "Autostart is currently turned off (run `mosaico autostart enable` to turn it back on)."
        ),
        Ok(AutostartStatus::Empty) => say!(
            cx,
            "Autostart is currently enabled, but its entry is empty (run `mosaico autostart enable` or `mosaico autostart disable`)."
        ),
        Ok(AutostartStatus::Other {
            registered,
            expected,
        }) => say!(
            cx,
            "Autostart is currently enabled, but starts {registered} instead of {expected}."
        ),
        Ok(AutostartStatus::Missing(command)) => say!(
            cx,
            "Autostart is currently enabled, but \"{}\" is not a file.",
            command.program.display()
        ),
        Err(e) => {
            complain!(cx, "Error: {e}");
            return Exit::Failure;
        }
    }
    Exit::Success
}

/// Registers the mosaico executable this command runs from to start at
/// login, replacing any earlier registration and turning it on.
pub fn register(cx: &Context) -> mosaico_core::Result<()> {
    let exe = running_exe(cx)?;
    cx.autostart.enable(&LaunchCommand::autostart(exe))
}

/// How the registration compares with what [`register`] would register,
/// see [`AutostartStatus::read`].
pub fn check(cx: &Context) -> mosaico_core::Result<AutostartStatus> {
    AutostartStatus::read(cx.autostart, cx.files, || running_exe(cx))
}

/// The mosaico executable this command runs from.
fn running_exe(cx: &Context) -> mosaico_core::Result<PathBuf> {
    (cx.current_exe)().map_err(|e| Error::platform("find the mosaico executable", e))
}

#[cfg(test)]
#[path = "tests/autostart_tests.rs"]
mod tests;
