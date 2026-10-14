//! Starting mosaico when the user logs in.

use mosaico_core::Error;
use mosaico_core::system::LaunchCommand;

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
    if cx.autostart.is_enabled() {
        say!(cx, "Autostart is currently enabled.");
    } else {
        say!(cx, "Autostart is currently disabled.");
    }
    Exit::Success
}

/// Registers the mosaico executable this command runs from to start at
/// login, replacing any earlier registration.
pub fn register(cx: &Context) -> mosaico_core::Result<()> {
    let exe = (cx.current_exe)().map_err(|e| Error::platform("find the mosaico executable", e))?;
    cx.autostart.enable(&LaunchCommand::autostart(exe))
}

#[cfg(test)]
#[path = "tests/autostart_tests.rs"]
mod tests;
