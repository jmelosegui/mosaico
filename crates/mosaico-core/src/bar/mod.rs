//! What the status bar shows, and the contracts that put it on screen.
//!
//! Core decides what every bar shows. [`BarContent::build`] turns the bar
//! config, a monitor's [`BarState`] and the [`SystemReadings`] into the
//! items to draw: which widgets appear, in what order, with what text and
//! colors. The readings come from [`SystemInfo`], which each platform
//! implements by reporting raw values, so every platform shows the same
//! bar.

mod content;
mod cpu;
mod readings;
mod state;
mod system_info;
mod time;
mod widget_text;

pub use content::{BarContent, BarItem};
pub use cpu::CpuTracker;
pub use readings::SystemReadings;
pub use state::BarState;
pub use system_info::{CpuTimes, NowPlaying, SystemInfo};
pub use time::LocalTime;

#[cfg(test)]
#[path = "tests/fakes.rs"]
mod fakes;
