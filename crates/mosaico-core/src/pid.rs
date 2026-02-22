use std::fs;
use std::path::PathBuf;

use crate::WindowResult;

/// Returns the path to the Mosaico data directory.
///
/// Uses the platform's local data directory (e.g.
/// `%LOCALAPPDATA%\mosaico` on Windows, `~/.local/share/mosaico`
/// on Linux). Creates the directory if it doesn't exist.
fn data_dir() -> WindowResult<PathBuf> {
    let base = dirs::data_local_dir().ok_or("could not determine local data directory")?;
    let dir = base.join("mosaico");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Returns the path to the PID file.
pub fn pid_path() -> WindowResult<PathBuf> {
    Ok(data_dir()?.join("mosaico.pid"))
}

/// Writes the current process's PID to the PID file.
///
/// Called when the daemon starts. The PID file allows the CLI to detect
/// a running daemon even if the IPC check fails, and to forcibly
/// kill a stuck daemon process.
pub fn write_pid_file() -> WindowResult<()> {
    let path = pid_path()?;
    let pid = std::process::id();
    fs::write(&path, pid.to_string())?;
    Ok(())
}

/// Reads the PID from the PID file, if it exists.
pub fn read_pid_file() -> WindowResult<Option<u32>> {
    let path = pid_path()?;

    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&path)?;
    let pid: u32 = contents
        .trim()
        .parse()
        .map_err(|e| format!("invalid PID file contents: {e}"))?;

    Ok(Some(pid))
}

/// Removes the PID file.
///
/// Called when the daemon shuts down cleanly.
pub fn remove_pid_file() -> WindowResult<()> {
    let path = pid_path()?;

    if path.exists() {
        fs::remove_file(&path)?;
    }

    Ok(())
}
