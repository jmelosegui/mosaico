//! Paused indicator widget — visible only while mosaico hotkeys are paused.

use super::BarState;

/// Returns "PAUSED" when hotkeys are paused, empty string otherwise.
///
/// The widget is hidden by `should_skip` when `state.paused` is false,
/// so this function will only be called when paused is true.
pub fn text(_state: &BarState) -> String {
    "PAUSED".to_string()
}
