//! Layout widget — displays the current layout name and monocle state.

use super::BarState;

/// Returns the layout indicator text (e.g. "BSP" or "BSP | M").
pub fn text(state: &BarState) -> String {
    if state.monocle {
        format!("{} | M", state.layout_name)
    } else {
        state.layout_name.clone()
    }
}
