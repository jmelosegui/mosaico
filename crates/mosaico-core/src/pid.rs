use std::fs;
use std::path::PathBuf;

use crate::{Instance, WindowResult};

/// Returns the path to the PID file of this process's [`Instance`],
/// creating its data directory if needed.
///
/// For the default instance that is `%LOCALAPPDATA%\mosaico\mosaico.pid`.
pub fn pid_path() -> WindowResult<PathBuf> {
    let instance = Instance::current()?;
    fs::create_dir_all(instance.data_dir())?;
    Ok(instance.pid_path())
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
