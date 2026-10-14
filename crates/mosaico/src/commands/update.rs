//! Self-update command: downloads and installs the latest release.
//!
//! Checks GitHub for a newer version, stops the daemon if running,
//! downloads this platform's release archive, has the platform replace
//! the running binary with the one in it, and restarts the daemon if it
//! was running before the update.

use std::fmt::Display;
use std::io::Read;
use std::time::Duration;

use mosaico_core::http::HttpClient;
use mosaico_core::update;

use super::context::{Context, Exit, complain, say};

const GITHUB_HOST: &str = "github.com";
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(60);

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

    // 2. Was the daemon running?
    let daemon_was_running = cx.daemon.is_running();

    // 3. Stop daemon if running.
    if daemon_was_running {
        say!(cx, "Stopping daemon...");
        super::stop::execute(cx);
    }

    // 4. Download this platform's release archive.
    say!(cx, "Downloading {tag}...");
    let path = format!(
        "/jmelosegui/mosaico/releases/download/{tag}/{}",
        cx.installer.asset_name()
    );
    let zip_bytes = match download_release(cx.http, &path) {
        Ok(bytes) => bytes,
        Err(e) => return fail(cx, e, daemon_was_running),
    };

    // 5. Extract the binary from the zip.
    let exe_bytes = match extract_binary(&zip_bytes, cx.installer.binary_name()) {
        Ok(bytes) => bytes,
        Err(e) => return fail(cx, e, daemon_was_running),
    };

    // 6. Replace the running binary.
    let current_exe = match (cx.current_exe)() {
        Ok(p) => p,
        Err(e) => {
            let reason = format!("cannot determine current exe path: {e}");
            return fail(cx, reason, daemon_was_running);
        }
    };

    say!(cx, "Installing to {}...", current_exe.display());
    if let Err(e) = cx.installer.replace(&current_exe, &exe_bytes) {
        return fail(cx, e, daemon_was_running);
    }

    if is_update {
        say!(cx, "Updated successfully: v{local} -> {tag}.");
    } else {
        say!(cx, "Reinstalled v{local}.");
    }

    // 7. Restart daemon if it was running.
    if daemon_was_running {
        say!(cx, "Restarting daemon...");
        return super::start::execute(cx);
    }
    Exit::Success
}

/// Downloads the release zip at `path` from GitHub.
fn download_release(http: &dyn HttpClient, path: &str) -> Result<Vec<u8>, String> {
    http.get_bytes(GITHUB_HOST, path, DOWNLOAD_TIMEOUT)
        .map_err(|e| e.to_string())
}

/// Extracts the file named `binary` from a zip archive.
fn extract_binary(zip_bytes: &[u8], binary: &str) -> Result<Vec<u8>, String> {
    let reader = std::io::Cursor::new(zip_bytes);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| format!("invalid zip archive: {e}"))?;

    let wanted = binary.to_lowercase();
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("cannot read zip entry: {e}"))?;

        let name = file.name().to_lowercase();
        if name == wanted || name.ends_with(&format!("/{wanted}")) {
            let mut buf = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buf)
                .map_err(|e| format!("cannot extract {binary}: {e}"))?;
            return Ok(buf);
        }
    }

    Err(format!("{binary} not found in zip archive"))
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
