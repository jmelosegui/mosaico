//! CPU usage from the platform's processor time counters.

use super::{CpuTimes, SystemInfo};

/// Turns processor time counters into the share of time spent busy.
///
/// Usage is the change between two readings, so the tracker keeps the
/// last one. The bar keeps a tracker only while a CPU widget is
/// configured, and samples it once per refresh.
#[derive(Debug, Clone)]
pub struct CpuTracker {
    /// The last counters read, or `None` until a read succeeds.
    last: Option<CpuTimes>,
}

impl CpuTracker {
    /// Starts tracking from the counters as they are now.
    pub fn new(system: &dyn SystemInfo) -> Self {
        Self {
            last: system.cpu_times().ok(),
        }
    }

    /// Share of time the processors were busy since the last sample, or
    /// since [`new`](Self::new) for the first one, from 0 to 100.
    ///
    /// Reports 0 when the counters cannot be read, and keeps the last
    /// reading, so the next sample covers the gap.
    pub fn sample(&mut self, system: &dyn SystemInfo) -> u32 {
        let Ok(now) = system.cpu_times() else {
            return 0;
        };
        let usage = self.last.map_or(0, |last| busy_percent(last, now));
        self.last = Some(now);
        usage
    }
}

/// Share of the time between two readings that was not idle.
fn busy_percent(last: CpuTimes, now: CpuTimes) -> u32 {
    let idle = now.idle.saturating_sub(last.idle);
    let total = now.total.saturating_sub(last.total);
    if total == 0 {
        return 0;
    }
    (total.saturating_sub(idle) * 100 / total) as u32
}

#[cfg(test)]
#[path = "tests/cpu_tests.rs"]
mod tests;
