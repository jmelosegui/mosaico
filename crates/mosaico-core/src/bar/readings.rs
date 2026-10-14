//! The system values the widgets show, read once per refresh.

use crate::config::bar::{BarConfig, WidgetConfig};

use super::{CpuTracker, LocalTime, NowPlaying, SystemInfo};

/// What the system widgets show, read once per refresh so every bar
/// shows the same values.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SystemReadings {
    /// Share of time the processors were busy, from 0 to 100.
    pub cpu_usage: u32,
    /// Share of physical memory in use, from 0 to 100.
    pub memory_usage: u32,
    /// The media that is playing, if any.
    pub now_playing: Option<NowPlaying>,
    /// The local time.
    pub time: LocalTime,
}

impl SystemReadings {
    /// Reads what the enabled widgets of `config` show.
    ///
    /// Memory and media are only read when an enabled widget shows them,
    /// and CPU usage only comes from `cpu`, which the bar keeps while a
    /// CPU widget is configured. A reading that fails shows as 0, or as
    /// nothing playing.
    pub fn read(system: &dyn SystemInfo, config: &BarConfig, cpu: Option<&mut CpuTracker>) -> Self {
        let memory_usage = if shows(config, |w| matches!(w, WidgetConfig::Ram { .. })) {
            system.memory_usage().unwrap_or(0)
        } else {
            0
        };
        let now_playing = if shows(config, |w| matches!(w, WidgetConfig::Media { .. })) {
            system.now_playing().ok().flatten()
        } else {
            None
        };
        Self {
            cpu_usage: cpu.map_or(0, |tracker| tracker.sample(system)),
            memory_usage,
            now_playing,
            time: system.local_time(),
        }
    }
}

/// Whether `config` has an enabled widget, on either side, that `kind`
/// accepts.
fn shows(config: &BarConfig, kind: impl Fn(&WidgetConfig) -> bool) -> bool {
    config
        .left
        .iter()
        .chain(&config.right)
        .any(|w| w.enabled() && kind(w))
}

#[cfg(test)]
#[path = "tests/readings_tests.rs"]
mod tests;
