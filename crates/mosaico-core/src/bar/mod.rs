//! What the status bar shows, and the contracts that put it on screen.
//!
//! Core decides what every bar shows.
//! [`BarContent::build`](crate::bar::BarContent::build) turns the bar
//! config, a monitor's [`BarState`](crate::bar::BarState) and the
//! [`SystemReadings`](crate::bar::SystemReadings) into the items to draw:
//! which widgets appear, in what order, with what text and colors.
//! [`BarManager`](crate::bar::BarManager) decides which monitors get a
//! bar and refreshes them. A platform only reports raw readings through
//! [`SystemInfo`](crate::bar::SystemInfo) and draws through
//! [`BarRenderer`](crate::bar::BarRenderer), so every platform shows the
//! same bar.
//!
//! Core also decides what the pointer does on a bar. A platform reports
//! presses, releases, wheel turns and the pointer leaving to a
//! [`PointerSink`](crate::bar::PointerSink), with positions in bar
//! coordinates, and says where it drew each item as an
//! [`ItemRegion`](crate::bar::ItemRegion). The manager finds what was
//! clicked with [`hit`](crate::bar::hit), adds the wheel's turns up with
//! [`WheelSteps`](crate::bar::WheelSteps), and returns the actions they
//! ask for. What a click on a widget runs is decided from the bar config
//! by [`WidgetClicks`](crate::bar::WidgetClicks).

mod clicks;
mod content;
mod cpu;
mod manager;
mod pointer;
mod press;
mod readings;
mod region;
mod renderer;
mod state;
mod system_info;
mod time;
mod wheel;
mod widget_text;

pub use clicks::{ClickProblem, RejectedClick, WidgetClicks};
pub use content::{BarContent, BarItem};
pub use cpu::CpuTracker;
pub use manager::BarManager;
pub use pointer::{BarId, Button, PointerEvent, PointerSink};
pub use press::Press;
pub use readings::SystemReadings;
pub use region::{BarSide, BarTarget, ItemRegion, WidgetRef, hit};
pub use renderer::{BarRenderer, BarSurface};
pub use state::BarState;
pub use system_info::{CpuTimes, NowPlaying, SystemInfo};
pub use time::LocalTime;
pub use wheel::{WHEEL_QUIET, WheelSteps};

#[cfg(test)]
#[path = "tests/fakes.rs"]
pub(crate) mod fakes;
