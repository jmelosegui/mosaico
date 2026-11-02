//! What the installer package runs around changing mosaico's files.
//!
//! An installer replaces or removes files, but only mosaico knows how to
//! stop its daemon so that it puts back the windows it hid, which ending
//! the process would not. The package runs this hidden command with
//! one [`Hook`] at each step:
//!
//! - before an upgrade or reinstall, from the old executable: stop the
//!   daemon, noting whether it ran, and move a running settings app
//!   aside, as its file cannot be replaced while it runs;
//! - after an install, from the new executable: start the daemon again if
//!   the hook before stopped it;
//! - before an uninstall: stop the daemon, delete what earlier updates
//!   set aside, move a running settings app aside, and remove the
//!   autostart entry when it starts this mosaico.
//!
//! The installer goes on whatever a hook reports, so a hook does what it
//! can and says what failed.

use mosaico_core::system::{AutostartStatus, LaunchCommand};

use super::context::{Context, Exit, complain, say};

/// The step of an install a hook runs at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Hook {
    /// Before the files of the installed version are replaced.
    BeforeUpgrade,
    /// After the new version's files are in place.
    AfterInstall,
    /// Before the files are removed for good.
    BeforeUninstall,
}

/// Runs `hook`.
pub fn execute(cx: &mut Context, hook: Hook) -> Exit {
    match hook {
        Hook::BeforeUpgrade => before_upgrade(cx),
        Hook::AfterInstall => after_install(cx),
        Hook::BeforeUninstall => before_uninstall(cx),
    }
}

/// Stops the daemon, noting that it should start again if it ran, and
/// moves a running settings app aside.
fn before_upgrade(cx: &mut Context) -> Exit {
    let was_running = cx.daemon.is_running();
    let stopped = super::stop::execute(cx);
    let noted = if was_running && stopped == Exit::Success {
        let noted = cx.restart.set();
        report(cx, noted, "remember to restart the daemon")
    } else {
        Exit::Success
    };
    let set_aside = set_settings_aside(cx);
    worst([stopped, noted, set_aside])
}

/// Starts the daemon if the hook before the install stopped it.
fn after_install(cx: &mut Context) -> Exit {
    match cx.restart.take() {
        Ok(true) => super::start::quietly(cx),
        Ok(false) => Exit::Success,
        Err(e) => report(cx, Err(e), "check whether to restart the daemon"),
    }
}

/// Stops the daemon, moves a running settings app aside and removes the
/// autostart entry when it starts this mosaico, not another copy.
fn before_uninstall(cx: &mut Context) -> Exit {
    let stopped = super::stop::execute(cx);
    // Nothing starts after an uninstall to delete what earlier updates
    // set aside, so the hook does, before it sets aside anything else.
    // One still in use stays, as no hook runs after the removal.
    if let Ok(exe) = (cx.current_exe)() {
        let _ = cx.installer.remove_leftovers(&exe);
    }
    let set_aside = set_settings_aside(cx);
    let autostart = match registers_this(cx) {
        Ok(true) => super::autostart::disable(cx),
        Ok(false) => Exit::Success,
        Err(e) => report(cx, Err(e), "check autostart"),
    };
    // Nothing starts after an uninstall, so a note left by a failed
    // upgrade would only start a later install's daemon unasked.
    let forgotten = cx.restart.take().map(drop);
    let forgotten = report(cx, forgotten, "clear the restart note");
    worst([stopped, set_aside, autostart, forgotten])
}

/// Whether the autostart entry runs this executable, with any arguments
/// and whether or not the user turned it off: an entry left behind would
/// run a program that is gone, while another copy's entry still works.
fn registers_this(cx: &Context) -> mosaico_core::Result<bool> {
    let Some(registered) = cx.autostart.registered_command()? else {
        return Ok(false);
    };
    let exe = super::autostart::running_exe(cx)?;
    let status = AutostartStatus::of(
        Some(LaunchCommand::autostart(registered.program)),
        &exe,
        cx.files,
    );
    Ok(status == AutostartStatus::Enabled)
}

/// Renames the settings app beside this executable aside when it runs,
/// which lets the installer replace or remove its file while the user
/// still has it open, with any unsaved edits.
fn set_settings_aside(cx: &mut Context) -> Exit {
    let exe = match (cx.current_exe)() {
        Ok(exe) => exe,
        Err(e) => {
            complain!(cx, "Error: cannot find the mosaico executable: {e}");
            return Exit::Failure;
        }
    };
    let settings = exe.with_file_name(cx.installer.settings_name());
    match cx.installer.set_aside_if_running(&settings) {
        Ok(true) => {
            say!(cx, "Moved the running settings app aside.");
            Exit::Success
        }
        Ok(false) => Exit::Success,
        Err(e) => report(cx, Err(e), "move the settings app aside"),
    }
}

/// Says why `result` failed, if it did.
fn report(cx: &mut Context, result: mosaico_core::Result<()>, what: &str) -> Exit {
    match result {
        Ok(()) => Exit::Success,
        Err(e) => {
            complain!(cx, "Error: could not {what}: {e}");
            Exit::Failure
        }
    }
}

/// A failure if any step failed.
fn worst(exits: impl IntoIterator<Item = Exit>) -> Exit {
    if exits.into_iter().any(|exit| exit == Exit::Failure) {
        Exit::Failure
    } else {
        Exit::Success
    }
}

#[cfg(test)]
#[path = "tests/installer_hook_tests.rs"]
mod tests;
