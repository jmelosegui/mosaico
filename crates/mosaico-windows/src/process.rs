use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE, TerminateProcess,
};

/// Checks whether a process with the given PID is still alive.
///
/// Uses `OpenProcess` with minimal access rights. If the handle can be
/// opened, the process exists. This is used to detect stale PID files
/// left behind when the daemon is killed without a clean shutdown.
pub fn is_process_alive(pid: u32) -> bool {
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
/// elevated windows (which cannot be repositioned via UIPI).
pub fn is_current_process_elevated() -> bool {
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

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
/// process is still alive (e.g. the IPC thread crashed).
pub fn kill_process(pid: u32) -> bool {
    // SAFETY: OpenProcess with PROCESS_TERMINATE grants permission to
    // call TerminateProcess. Both handles are closed on drop/error.
    let result = unsafe { OpenProcess(PROCESS_TERMINATE, false, pid) };

    match result {
        Ok(handle) => {
            let killed = unsafe { TerminateProcess(handle, 1) };
            unsafe {
                let _ = CloseHandle(handle);
            }
            killed.is_ok()
        }
        Err(_) => false,
    }
}
