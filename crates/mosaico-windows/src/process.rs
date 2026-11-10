//! Processes on Windows: [`ProcessControl`](mosaico_core::system::ProcessControl)
//! for the CLI, and whether mosaico itself runs elevated, for the desktop.

use std::ffi::OsStr;
use std::path::Path;

use mosaico_core::Result;
use mosaico_core::system::{ProcessControl, RunningProcess};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, CreateProcessW, OpenProcess, PROCESS_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE, STARTF_USESTDHANDLES, STARTUPINFOW,
    TerminateProcess,
};
use windows::core::{PCWSTR, PWSTR};

use crate::command_line::command_line;
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

/// Starts `program` with `args`, detached, and returns its pid.
///
/// - `CREATE_NEW_PROCESS_GROUP`: the program gets its own process
///   group, so Ctrl+C in the terminal that started it does not reach it.
/// - `CREATE_NO_WINDOW`: a console program gets no console window.
/// - No handle of this process is inherited, and the standard streams
///   are null, so the program never holds open a pipe this process
///   writes to: an installer that runs `mosaico installer-hook` reads
///   its output until the pipe closes, and would otherwise wait for the
///   daemon the hook starts.
///
/// A `program` without a folder is looked up as Windows looks up a
/// command: this program's folder, the system folders, then `PATH`.
pub(crate) fn spawn_detached(
    program: &OsStr,
    args: &[&OsStr],
    operation: &'static str,
) -> Result<u32> {
    let mut line = command_line(program, args);
    let startup = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        dwFlags: STARTF_USESTDHANDLES,
        ..Default::default()
    };
    let mut process = PROCESS_INFORMATION::default();
    // SAFETY: `line` is a writable, nul-terminated UTF-16 buffer that
    // outlives the call, as CreateProcessW requires; `startup` and
    // `process` are valid for the call. Both handles it returns are
    // closed below.
    unsafe {
        CreateProcessW(
            PCWSTR::null(),
            Some(PWSTR(line.as_mut_ptr())),
            None,
            None,
            false,
            CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW,
            None,
            PCWSTR::null(),
            &startup,
            &mut process,
        )
    }
    .or_platform(operation)?;
    // SAFETY: CreateProcessW succeeded, so both handles are open and
    // owned here; the process keeps running once they are closed.
    unsafe {
        let _ = CloseHandle(process.hThread);
        let _ = CloseHandle(process.hProcess);
    }
    Ok(process.dwProcessId)
}

/// [`ProcessControl`] on Windows.
pub struct WindowsProcesses;

impl ProcessControl for WindowsProcesses {
    /// Re-runs the current executable with the hidden `daemon`
    /// subcommand, detached and with its standard streams closed.
    fn launch_daemon(&self) -> Result<u32> {
        let exe = std::env::current_exe().or_platform("find the mosaico executable")?;
        spawn_detached(exe.as_os_str(), &[OsStr::new("daemon")], "start the daemon")
    }

    /// Runs `program` detached, with its standard streams closed.
    fn launch(&self, program: &Path) -> Result<u32> {
        spawn_detached(program.as_os_str(), &[], "start the program")
    }

    fn is_alive(&self, pid: u32) -> bool {
        is_process_alive(pid)
    }

    fn kill(&self, pid: u32) -> Result<()> {
        kill_process(pid)
    }

    fn processes(&self) -> Vec<RunningProcess> {
        crate::process_list::processes()
    }
}

#[cfg(test)]
#[path = "process_tests.rs"]
mod tests;
