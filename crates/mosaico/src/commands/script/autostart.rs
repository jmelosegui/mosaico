//! `mosaico script autostart`: starting a script when the user signs in.

use mosaico_core::system::LaunchCommand;

use super::paths::Folders;
use super::run::log_path;
use crate::commands::context::{Context, Exit, complain, say};

/// Starts the script called `name` at every sign-in, in the background.
pub fn enable(cx: &mut Context, name: &str) -> Exit {
    if !Folders::new(cx.config.dir()).script(name).exists() {
        complain!(
            cx,
            "Error: there is no script called {name}. See `mosaico script list`."
        );
        return Exit::Failure;
    }
    let exe = match (cx.current_exe)() {
        Ok(exe) => exe,
        Err(e) => {
            complain!(cx, "Error: could not find the mosaico executable: {e}");
            return Exit::Failure;
        }
    };
    match cx
        .script_autostart
        .enable(name, &LaunchCommand::script(exe, name))
    {
        Ok(()) => {
            let log = log_path(cx.config.dir(), name);
            say!(
                cx,
                "{name} starts when you sign in. It writes to {}",
                log.display()
            );
            Exit::Success
        }
        Err(e) => {
            complain!(cx, "Error: could not set {name} to start at sign-in: {e}");
            Exit::Failure
        }
    }
}

/// Stops the script called `name` starting at sign-in. It keeps running
/// if it is.
pub fn disable(cx: &mut Context, name: &str) -> Exit {
    match cx.script_autostart.disable(name) {
        Ok(()) => {
            say!(cx, "{name} no longer starts when you sign in.");
            Exit::Success
        }
        Err(e) => {
            complain!(cx, "Error: {e}");
            Exit::Failure
        }
    }
}
