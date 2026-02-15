use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

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
