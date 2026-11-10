//! When mosaico's own bar draws again on its own, with no change in the
//! state it shows.

use std::time::Duration;

use crate::config::bar::{BarConfig, WidgetConfig};

use super::LocalTime;

/// How often the machine is read again while a widget shows a reading of
/// it, or a clock shows seconds.
pub const READING_INTERVAL: Duration = Duration::from_secs(1);

/// How long after a draw that read the time as `time` the bars of
/// `config` show something new on their own, or `None` when nothing they
/// show changes but with the state.
///
/// CPU, memory and media are read again every [`READING_INTERVAL`], as is
/// a clock that shows seconds. A clock or date without seconds is drawn
/// again as the minute turns, so a bar that only shows the time wakes the
/// daemon once a minute, and one that shows neither never does.
pub fn refresh_in(config: &BarConfig, time: LocalTime) -> Option<Duration> {
    let next_minute = Duration::from_secs(60 - u64::from(time.second.min(59)));
    (config.left.iter().chain(&config.right))
        .filter(|widget| widget.enabled())
        .filter_map(|widget| match widget {
            WidgetConfig::Cpu { .. } | WidgetConfig::Ram { .. } | WidgetConfig::Media { .. } => {
                Some(READING_INTERVAL)
            }
            WidgetConfig::Clock { format, .. } | WidgetConfig::Date { format, .. } => {
                Some(if shows_seconds(format) {
                    READING_INTERVAL
                } else {
                    next_minute
                })
            }
            _ => None,
        })
        .min()
}

/// Whether `format` shows the seconds, with `%S` outside a `%%`.
fn shows_seconds(format: &str) -> bool {
    let mut chars = format.chars();
    while let Some(c) = chars.next() {
        if c == '%' && chars.next() == Some('S') {
            return true;
        }
    }
    false
}

#[cfg(test)]
#[path = "tests/refresh_tests.rs"]
mod tests;
