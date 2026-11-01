//! The text each widget shows, and when it shows at all.

use crate::config::bar::WidgetConfig;

use super::{BarIcon, BarState, NowPlaying, SystemReadings};

/// Whether `widget` shows at all.
///
/// A disabled widget never shows. The update, media and paused widgets
/// only show while there is an update, something playing, or the hotkeys
/// are paused.
pub(super) fn is_shown(widget: &WidgetConfig, state: &BarState, readings: &SystemReadings) -> bool {
    if !widget.enabled() {
        return false;
    }
    match widget {
        WidgetConfig::Update { .. } => !state.update_text.is_empty(),
        WidgetConfig::Media { .. } => !playing(readings).is_empty(),
        WidgetConfig::Paused { .. } => state.paused,
        _ => true,
    }
}

/// The drawing of `widget`'s icon, when its glyph has one.
pub(super) fn icon(widget: &WidgetConfig) -> Option<BarIcon> {
    BarIcon::from_glyph(widget.icon())
}

/// What the pill of `widget` reads: its icon, unless that is drawn in
/// lines (see [`icon`]), then its text.
pub(super) fn label(widget: &WidgetConfig, state: &BarState, readings: &SystemReadings) -> String {
    let icon = if icon(widget).is_some() {
        ""
    } else {
        widget.icon()
    };
    let text = text(widget, state, readings);
    if icon.is_empty() {
        text
    } else if text.is_empty() {
        icon.to_string()
    } else {
        format!("{icon} {text}")
    }
}

/// The text of `widget`, without its icon. The workspaces and active
/// window widgets draw their own content, so they have none.
pub(super) fn text(widget: &WidgetConfig, state: &BarState, readings: &SystemReadings) -> String {
    match widget {
        WidgetConfig::Workspaces { .. } | WidgetConfig::ActiveWindow { .. } => String::new(),
        WidgetConfig::Layout { .. } => layout(state),
        WidgetConfig::Clock { format, .. } | WidgetConfig::Date { format, .. } => {
            readings.time.format(format)
        }
        WidgetConfig::Ram { .. } => format!("RAM: {}%", readings.memory_usage),
        WidgetConfig::Cpu { .. } => format!("CPU: {:>3}%", readings.cpu_usage),
        WidgetConfig::Update { .. } => state.update_text.clone(),
        WidgetConfig::Media { max_length, .. } => media(readings, *max_length),
        WidgetConfig::Paused { .. } => "PAUSED".to_string(),
    }
}

/// The layout name, followed by `| M` in monocle mode.
fn layout(state: &BarState) -> String {
    let name = state.layout.name();
    if state.monocle {
        format!("{name} | M")
    } else {
        name.to_string()
    }
}

/// What is playing, cut to `max_length` characters. A `max_length` of 0
/// leaves only the icon.
fn media(readings: &SystemReadings, max_length: usize) -> String {
    let playing = playing(readings);
    if playing.is_empty() || max_length == 0 {
        return String::new();
    }
    truncate(&playing, max_length)
}

/// "Artist - Title", leaving out a part that is empty.
fn playing(readings: &SystemReadings) -> String {
    let Some(NowPlaying { artist, title }) = &readings.now_playing else {
        return String::new();
    };
    match (artist.is_empty(), title.is_empty()) {
        (false, false) => format!("{artist} - {title}"),
        (false, true) => artist.clone(),
        (true, _) => title.clone(),
    }
}

/// Cuts `s` to `max` characters, ending in "..." when it was longer.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max.saturating_sub(3)).collect();
    format!("{truncated}...")
}

#[cfg(test)]
#[path = "tests/widget_text_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/media_text_tests.rs"]
mod media_tests;
