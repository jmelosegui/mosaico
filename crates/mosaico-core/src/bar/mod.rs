//! What the status bar shows, and the contracts that put it on screen.
//!
//! The system widgets read the machine through [`SystemInfo`], which each
//! platform implements. Turning those readings into what the widgets show,
//! such as a CPU percentage or a formatted time, happens here, so every
//! platform shows the same values.

mod cpu;
mod system_info;
mod time;

pub use cpu::CpuTracker;
pub use system_info::{CpuTimes, NowPlaying, SystemInfo};
pub use time::LocalTime;

#[cfg(test)]
#[path = "tests/fakes.rs"]
mod fakes;
