//! Starting mosaico when the user logs in.

use super::context::{Context, Exit, complain, say};

pub fn enable(cx: &mut Context) -> Exit {
    match cx.autostart.enable() {
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

#[cfg(test)]
#[path = "tests/autostart_tests.rs"]
mod tests;
