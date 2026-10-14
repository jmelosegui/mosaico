//! What the status bar shows, and the contracts that put it on screen.
//!
//! Core decides what every bar shows. [`BarContent::build`] turns the bar
//! config, a monitor's [`BarState`] and the [`SystemReadings`] into the
//! items to draw: which widgets appear, in what order, with what text and
//! colors. [`BarManager`] decides which monitors get a bar and refreshes
//! them. A platform only reports raw readings through [`SystemInfo`] and
//! draws through [`BarRenderer`], so every platform shows the same bar.

mod content;
mod cpu;
mod manager;
mod readings;
mod renderer;
mod state;
mod system_info;
mod time;
mod widget_text;

pub use content::{BarContent, BarItem};
pub use cpu::CpuTracker;
pub use manager::BarManager;
pub use readings::SystemReadings;
pub use renderer::{BarRenderer, BarSurface};
pub use state::BarState;
pub use system_info::{CpuTimes, NowPlaying, SystemInfo};
pub use time::LocalTime;

#[cfg(test)]
#[path = "tests/fakes.rs"]
mod fakes;
