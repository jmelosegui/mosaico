use std::fs;
use std::path::PathBuf;

use crate::WindowResult;

/// Returns the path to the Mosaico data directory.
///
/// On Windows: `%LOCALAPPDATA%\mosaico`
/// Creates the directory if it doesn't exist.
fn data_dir() -> WindowResult<PathBuf> {
    // LOCALAPPDATA is the standard location for per-user application data
    // on Windows (e.g. C:\Users\username\AppData\Local).
    let base =
        std::env::var("LOCALAPPDATA").map_err(|_| "LOCALAPPDATA environment variable not set")?;

    let dir = PathBuf::from(base).join("mosaico");
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
/// a running daemon even if the named pipe check fails, and to forcibly
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
