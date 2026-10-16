//! What the bar's system widgets read from the machine.

use crate::Result;

use super::LocalTime;

/// Readings of the machine for the bar's system widgets.
///
/// Platforms report raw values. Core turns them into what the widgets
/// show, so every platform shows the same thing.
///
/// # Contract for platform implementations
///
/// - Every method only reads. None may change anything.
/// - The daemon calls these on its own thread at least once a second, so
///   each must return at once. Never wait on another process or on a
///   request that can take long.
/// - [`cpu_times`](Self::cpu_times) counts processor time from a fixed
///   point, such as boot, summed over every processor. Both counters only
///   grow, share one unit, and `idle` never exceeds `total`. Core turns
///   two readings into a percentage, see [`CpuTracker`](super::CpuTracker).
/// - [`memory_usage`](Self::memory_usage) is the share of physical memory
///   in use, from 0 to 100, as the OS itself reports it.
/// - [`now_playing`](Self::now_playing) returns `Ok(None)` when nothing is
///   playing, and [`Error::Unsupported`](crate::Error::Unsupported) when
///   the platform cannot tell. The media widget stays hidden either way.
/// - [`local_time`](Self::local_time) cannot fail. A platform that cannot
///   read the user's time zone reports UTC.
pub trait SystemInfo {
    /// Processor time spent idle, and in total.
    fn cpu_times(&self) -> Result<CpuTimes>;

    /// Share of physical memory in use, from 0 to 100.
    fn memory_usage(&self) -> Result<u32>;

    /// The media that is playing, if any.
    fn now_playing(&self) -> Result<Option<NowPlaying>>;

    /// The current time in the user's time zone.
    fn local_time(&self) -> LocalTime;
}

/// Processor time counted from a fixed point, summed over every
/// processor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CpuTimes {
    /// Time spent idle.
    pub idle: u64,
    /// Time spent idle or busy.
    pub total: u64,
}

/// What is playing, as the media widget shows it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NowPlaying {
    /// Who made it. Empty when the player does not say.
    pub artist: String,
    /// What it is called.
    pub title: String,
}
