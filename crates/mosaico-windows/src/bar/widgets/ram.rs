//! RAM widget — displays system memory usage.

use std::mem;

use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

/// Returns the current RAM usage string (e.g. "RAM: 24%").
pub fn text() -> String {
    format!("RAM: {}%", usage_percent())
}

/// Queries the OS for current memory load as a percentage (0–100).
fn usage_percent() -> u32 {
    unsafe {
        let mut status = MEMORYSTATUSEX {
            dwLength: mem::size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };
        if GlobalMemoryStatusEx(&mut status).is_ok() {
            status.dwMemoryLoad
        } else {
            0
        }
    }
}
