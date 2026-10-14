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
        Ok(AutostartStatus::Other(command)) => say!(
            cx,
            "Autostart is currently enabled, but starts {command} instead of this mosaico."
        ),
        Ok(AutostartStatus::Missing(command)) => say!(
            cx,
            "Autostart is currently enabled, but \"{}\" does not exist.",
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
/// login, replacing any earlier registration.
pub fn register(cx: &Context) -> mosaico_core::Result<()> {
    let exe = running_exe(cx)?;
    cx.autostart.enable(&LaunchCommand::autostart(exe))
}

/// How the registration compares with what [`register`] would register.
pub fn check(cx: &Context) -> mosaico_core::Result<AutostartStatus> {
    let exe = running_exe(cx)?;
    let registered = cx.autostart.registered_command()?;
    Ok(AutostartStatus::of(registered, &exe))
}

/// The mosaico executable this command runs from.
fn running_exe(cx: &Context) -> mosaico_core::Result<PathBuf> {
    (cx.current_exe)().map_err(|e| Error::platform("find the mosaico executable", e))
}

#[cfg(test)]
#[path = "tests/autostart_tests.rs"]
mod tests;
