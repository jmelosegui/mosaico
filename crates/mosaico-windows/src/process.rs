//! Processes on Windows: [`ProcessControl`](mosaico_core::system::ProcessControl)
//! for the CLI, and whether mosaico itself runs elevated, for the desktop.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

use mosaico_core::Result;
use mosaico_core::system::ProcessControl;
use windows::Win32::Foundation::{
    CloseHandle, HANDLE_FLAG_INHERIT, HANDLE_FLAGS, SetHandleInformation,
};
use windows::Win32::System::Console::{
    GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE, TerminateProcess,
};

use crate::error::OrPlatform;

/// Checks whether a process with the given PID is still alive.
///
/// Uses `OpenProcess` with minimal access rights. If the handle can be
/// opened, the process exists. This is used to detect stale PID files
/// left behind when the daemon is killed without a clean shutdown.
fn is_process_alive(pid: u32) -> bool {
    // SAFETY: OpenProcess attempts to open an existing process.
    // PROCESS_QUERY_LIMITED_INFORMATION is the least-privilege access
    // right that still lets us confirm the process exists.
    let result = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) };

    match result {
        Ok(handle) => {
            // SAFETY: We only opened the handle to check existence,
            // so we close it immediately.
            unsafe {
                let _ = CloseHandle(handle);
            }
            true
        }
        Err(_) => false,
    }
}

/// Returns whether the current process is running elevated (Administrator).
///
/// Called once at startup so the tiling manager knows whether to skip
/// elevated windows (which cannot be repositioned via UIPI). That is how
/// Windows decides which windows mosaico may move, so it stays inside
/// this crate, with the desktop that leaves those windows out, rather
/// than becoming a core contract.
pub(crate) fn is_current_process_elevated() -> bool {
    use windows::Win32::Security::{
        GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    // SAFETY: Queries the current process token for elevation status.
    // The token handle is closed before returning.
    unsafe {
        let mut token = windows::Win32::Foundation::HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut returned = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        );
        let _ = CloseHandle(token);

        ok.is_ok() && elevation.TokenIsElevated != 0
    }
}

/// Forcibly terminates a process by PID.
///
/// Used as a fallback when the daemon's IPC pipe is gone but the
/// process is still alive (e.g. the IPC thread crashed). Fails when the
/// process cannot be opened for termination or refuses to end.
fn kill_process(pid: u32) -> Result<()> {
    // SAFETY: OpenProcess with PROCESS_TERMINATE grants permission to
    // call TerminateProcess. The handle is closed below.
    let handle = unsafe { OpenProcess(PROCESS_TERMINATE, false, pid) }
        .or_platform("open the process to end it")?;

    // SAFETY: TerminateProcess on a valid handle with PROCESS_TERMINATE.
    let killed = unsafe { TerminateProcess(handle, 1) };
    // SAFETY: CloseHandle releases the process handle.
    unsafe {
        let _ = CloseHandle(handle);
    }
    killed.or_platform("end the process")
}

/// Process creation flags for launching a fully detached daemon.
///
/// `CREATE_NEW_PROCESS_GROUP` (0x200): the daemon gets its own process
/// group, so Ctrl+C in the CLI terminal does not reach it.
///
/// `CREATE_NO_WINDOW` (0x08000000): the daemon gets no console, which
/// also keeps it from inheriting the parent's console handles. Inherited
/// handles are what make `cmd.output()` hang in tests.
const DETACH_FLAGS: u32 = 0x08000000 | 0x00000200;

/// Stops children from inheriting this process's own standard handles.
///
/// A detached child gets null standard streams, but Windows still hands
/// it every inheritable handle of its parent. When this process's output
/// is a pipe, as when an installer runs `mosaico installer-hook` and
/// reads its output until the pipe closes, the daemon it starts would
/// hold that pipe open for as long as it runs, and the installer would
/// wait for it. Children that inherit these streams on purpose are not
/// affected: Rust duplicates the handle it hands them.
fn keep_standard_handles_to_this_process() {
    for stream in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        // SAFETY: GetStdHandle only reads this process's handle table.
        let Ok(handle) = (unsafe { GetStdHandle(stream) }) else {
            continue;
        };
        if handle.is_invalid() {
            continue;
        }
        // SAFETY: `handle` is one of this process's standard handles,
        // and only its inherit flag changes. A console or missing handle
        // that has no such flag fails harmlessly.
        let _ = unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0)) };
    }
}

/// Starts `command` detached, with its standard streams closed and none
/// of this process's handles, and returns its pid.
fn spawn_detached(mut command: Command, operation: &'static str) -> Result<u32> {
    keep_standard_handles_to_this_process();
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(DETACH_FLAGS)
        .spawn()
        .or_platform(operation)?;
    Ok(child.id())
}

/// [`ProcessControl`] on Windows.
pub struct WindowsProcesses;

impl ProcessControl for WindowsProcesses {
    /// Re-runs the current executable with the hidden `daemon`
    /// subcommand, detached and with its standard streams closed.
    fn launch_daemon(&self) -> Result<u32> {
        let exe = std::env::current_exe().or_platform("find the mosaico executable")?;
        let mut daemon = Command::new(exe);
        daemon.arg("daemon");
        spawn_detached(daemon, "start the daemon")
    }

    /// Runs `program` detached, with its standard streams closed.
    fn launch(&self, program: &Path) -> Result<u32> {
        spawn_detached(Command::new(program), "start the program")
    }

    fn is_alive(&self, pid: u32) -> bool {
        is_process_alive(pid)
    }

    fn kill(&self, pid: u32) -> Result<()> {
        kill_process(pid)
    }
}

#[cfg(test)]
#[path = "process_tests.rs"]
mod tests;
