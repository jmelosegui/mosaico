//! Self-update command: downloads and installs the latest release.
//!
//! Checks GitHub for a newer version, stops the daemon if running,
//! downloads this platform's release archive, has the platform replace
//! the settings app and the running binary with the ones in it, and
//! restarts the daemon if it was running before the update.

use std::fmt::Display;

use mosaico_core::update;

use super::context::{Context, Exit, complain, say};

/// Runs the full update cycle.
pub fn execute(cx: &mut Context, force: bool) -> Exit {
    // 1. Check for update.
    let _ = write!(cx.out, "Checking for updates...");
    let tag = match update::latest_tag(cx.http) {
        Ok(t) => t,
        Err(_) => {
            say!(cx, " failed.");
            complain!(cx, "Error: could not reach GitHub to check for updates.");
            return Exit::Failure;
        }
    };

    let local = env!("CARGO_PKG_VERSION");
    let is_update = update::is_newer_tag(&tag, local);

    if !is_update && !force {
        say!(cx, "");
        say!(cx, "You are already on the latest version (v{local}).");
        return Exit::Success;
    }

    if is_update {
        say!(cx, " v{local} -> {tag}.");
    } else {
        say!(cx, " reinstalling v{local}.");
    }

    // 2. Make sure this platform can install it before stopping anything.
    if let Err(e) = cx.installer.asset_name() {
        return fail(cx, e, false);
    }

    // 3. Stop daemon if running.
    let daemon_was_running = cx.daemon.is_running();
    if daemon_was_running {
        say!(cx, "Stopping daemon...");
        super::stop::execute(cx);
    }

    // 4. Download this platform's release and take the binaries out of it.
    say!(cx, "Downloading {tag}...");
    let binaries = match update::download_binaries(cx.http, cx.installer, &tag) {
        Ok(binaries) => binaries,
        Err(e) => return fail(cx, e, daemon_was_running),
    };

    // 5. Replace the settings app, then the running binary, so a failure
    // leaves the old mosaico in place.
    let current_exe = match (cx.current_exe)() {
        Ok(p) => p,
        Err(e) => {
            let reason = format!("cannot determine current exe path: {e}");
            return fail(cx, reason, daemon_was_running);
        }
    };
    let settings_exe = current_exe.with_file_name(cx.installer.settings_name());

    say!(cx, "Installing to {}...", current_exe.display());
    let installed = cx
        .installer
        .replace(&settings_exe, &binaries.settings)
        .and_then(|()| cx.installer.replace(&current_exe, &binaries.mosaico));
    if let Err(e) = installed {
        return fail(cx, e, daemon_was_running);
    }

    if is_update {
        say!(cx, "Updated successfully: v{local} -> {tag}.");
    } else {
        say!(cx, "Reinstalled v{local}.");
    }

    // 6. Restart daemon if it was running.
    if daemon_was_running {
        say!(cx, "Restarting daemon...");
        return super::start::execute(cx);
    }
    Exit::Success
}

/// Reports why the update failed and, when the daemon was stopped for
/// it, how to get it back.
fn fail(cx: &mut Context, reason: impl Display, daemon_was_running: bool) -> Exit {
    complain!(cx, "Error: {reason}");
    if daemon_was_running {
        complain!(cx, "The daemon was stopped before the update failed.");
        complain!(
            cx,
            "Run 'mosaico start' to restart with the current version."
        );
    }
    Exit::Failure
}

#[cfg(test)]
#[path = "tests/update_tests.rs"]
mod tests;
