//! CPU widget — tracks and displays system CPU usage.
//!
//! Uses `GetSystemTimes` to compute the delta between two samples.
//! The tracker must persist across ticks so the daemon owns it and
//! writes the result into [`BarState::cpu_usage`].

use windows::Win32::Foundation::FILETIME;
use windows::Win32::System::Threading::GetSystemTimes;

/// Tracks CPU usage across successive samples.
pub struct CpuTracker {
    prev_idle: u64,
    prev_total: u64,
}

impl CpuTracker {
    /// Creates a tracker and takes the first baseline sample.
    pub fn new() -> Self {
        let (idle, kernel, user) = system_times();
        Self {
            prev_idle: idle,
            prev_total: kernel + user,
        }
    }

    /// Samples current CPU times and returns usage as 0–100%.
    ///
    /// The first call after `new()` returns the usage since creation.
    pub fn sample(&mut self) -> u32 {
        let (idle, kernel, user) = system_times();
        let total = kernel + user;

        let delta_idle = idle.saturating_sub(self.prev_idle);
        let delta_total = total.saturating_sub(self.prev_total);

        self.prev_idle = idle;
        self.prev_total = total;

        if delta_total == 0 {
            return 0;
        }
        ((delta_total - delta_idle) * 100 / delta_total) as u32
    }
}

/// Returns the display text for a given usage percentage.
pub fn text(cpu_usage: u32) -> String {
    format!("CPU: {cpu_usage}%")
}

/// Queries idle, kernel, and user times as 64-bit tick counts.
fn system_times() -> (u64, u64, u64) {
    let mut idle = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();

    unsafe {
        let _ = GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user));
    }

    (ft_to_u64(idle), ft_to_u64(kernel), ft_to_u64(user))
}

fn ft_to_u64(ft: FILETIME) -> u64 {
    u64::from(ft.dwLowDateTime) | (u64::from(ft.dwHighDateTime) << 32)
}
