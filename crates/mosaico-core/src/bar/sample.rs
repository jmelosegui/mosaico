//! What a widget reads on a sample bar, to show how it will look before
//! it is on screen.

use crate::config::BarWidget;
use crate::config::bar::WidgetConfig;

use super::{BarState, LocalTime, NowPlaying, SystemReadings, widget_text};

/// The text `widget` shows, without its icon, on a bar whose hotkeys are
/// paused, with an update waiting, something playing, the processors 12%
/// busy, 48% of memory in use, and the time at 12:57:03 on Saturday 31
/// October 2026.
///
/// Every widget shows here, including those the bar only shows at times.
pub fn sample_text(widget: &BarWidget) -> String {
    let mut config = WidgetConfig::of_kind(widget.kind);
    if let (WidgetConfig::Clock { format, .. } | WidgetConfig::Date { format, .. }, Some(wanted)) =
        (&mut config, &widget.format)
    {
        format.clone_from(wanted);
    }
    let state = BarState {
        update_text: "v1.0.0 available".into(),
        paused: true,
        ..BarState::default()
    };
    let readings = SystemReadings {
        cpu_usage: 12,
        memory_usage: 48,
        now_playing: Some(NowPlaying {
            artist: "Artist".into(),
            title: "Song".into(),
        }),
        time: LocalTime {
            year: 2026,
            month: 10,
            day: 31,
            weekday: 6,
            hour: 12,
            minute: 57,
            second: 3,
        },
    };
    widget_text::text(&config, &state, &readings)
}

#[cfg(test)]
#[path = "tests/sample_tests.rs"]
mod tests;
