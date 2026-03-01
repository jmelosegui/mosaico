//! Self-update command: downloads and installs the latest release.
//!
//! Checks GitHub for a newer version, stops the daemon if running,
//! downloads the release zip, replaces the binary via rename-then-copy,
//! and restarts the daemon if it was running before the update.

use std::io::Read;

const GITHUB_HOST: &str = "github.com";
const DOWNLOAD_TIMEOUT_MS: i32 = 60_000;

/// Runs the full update cycle.
pub fn execute(force: bool) {
    // 1. Check for update.
    print!("Checking for updates...");
    let tag = match mosaico_windows::version_check::fetch_latest_tag() {
        Some(t) => t,
        None => {
            println!(" failed.");
            eprintln!("Error: could not reach GitHub to check for updates.");
            std::process::exit(1);
        }
    };

    let remote = tag.strip_prefix('v').unwrap_or(&tag);
    let local = env!("CARGO_PKG_VERSION");
    let is_update = mosaico_core::version::is_newer(remote, local);

    if !is_update && !force {
        println!();
        println!("You are already on the latest version (v{local}).");
        return;
    }

    if is_update {
        println!(" v{local} -> {tag}.");
    } else {
        println!(" reinstalling v{local}.");
    }

    // 2. Was the daemon running?
    let daemon_was_running = mosaico_windows::ipc::is_daemon_running();

    // 3. Stop daemon if running.
    if daemon_was_running {
        println!("Stopping daemon...");
        super::stop::execute();
    }

    // 4. Download the release zip.
    println!("Downloading {tag}...");
    let zip_bytes = match download_release(&tag) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: {e}");
            print_recovery_hint(daemon_was_running);
            std::process::exit(1);
        }
    };

    // 5. Extract mosaico.exe from the zip.
    let exe_bytes = match extract_exe(&zip_bytes) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: {e}");
            print_recovery_hint(daemon_was_running);
            std::process::exit(1);
        }
    };

    // 6. Replace the running binary.
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: cannot determine current exe path: {e}");
            print_recovery_hint(daemon_was_running);
            std::process::exit(1);
        }
    };

    println!("Installing to {}...", current_exe.display());
    if let Err(e) = install_exe(&current_exe, &exe_bytes) {
        eprintln!("Error: {e}");
        print_recovery_hint(daemon_was_running);
        std::process::exit(1);
    }

    if is_update {
        println!("Updated successfully: v{local} -> {tag}.");
    } else {
        println!("Reinstalled v{local}.");
    }

    // 7. Restart daemon if it was running.
    if daemon_was_running {
        println!("Restarting daemon...");
        super::start::execute();
    }
}

/// Downloads the release zip from GitHub.
fn download_release(tag: &str) -> Result<Vec<u8>, String> {
    let path = format!("/jmelosegui/mosaico/releases/download/{tag}/mosaico-windows-amd64.zip");
    mosaico_windows::http::get_bytes(GITHUB_HOST, &path, DOWNLOAD_TIMEOUT_MS)
}

/// Extracts `mosaico.exe` from a zip archive.
fn extract_exe(zip_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let reader = std::io::Cursor::new(zip_bytes);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| format!("invalid zip archive: {e}"))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("cannot read zip entry: {e}"))?;

        let name = file.name().to_lowercase();
        if name == "mosaico.exe" || name.ends_with("/mosaico.exe") {
            let mut buf = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buf)
                .map_err(|e| format!("cannot extract mosaico.exe: {e}"))?;
            return Ok(buf);
        }
    }

    Err("mosaico.exe not found in zip archive".into())
}

/// Replaces the current binary using rename-then-write.
///
/// Windows allows renaming a running `.exe` but not overwriting it.
/// We rename the current binary to `.old`, write the new one, then
/// try to delete the backup.
fn install_exe(current_exe: &std::path::Path, exe_bytes: &[u8]) -> Result<(), String> {
    let backup = current_exe.with_extension("exe.old");

    // Remove leftover backup from a previous update.
    let _ = std::fs::remove_file(&backup);

    // Rename the running binary out of the way.
    std::fs::rename(current_exe, &backup).map_err(|e| format!("cannot rename current exe: {e}"))?;

    // Write the new binary.
    if let Err(e) = std::fs::write(current_exe, exe_bytes) {
        // Attempt rollback.
        let _ = std::fs::rename(&backup, current_exe);
        return Err(format!("cannot write new exe: {e}"));
    }

    // Try to delete the backup (may fail if still locked).
    let _ = std::fs::remove_file(&backup);

    Ok(())
}

/// Prints a hint about recovery when the update fails after the
/// daemon was stopped.
fn print_recovery_hint(daemon_was_running: bool) {
    if daemon_was_running {
        eprintln!("The daemon was stopped before the update failed.");
        eprintln!("Run 'mosaico start' to restart with the current version.");
    }
}
