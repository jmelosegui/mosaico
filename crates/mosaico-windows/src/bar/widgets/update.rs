//! Update widget — displays an update notification when available.
//!
//! Currently a placeholder. A future version can check the GitHub
//! releases API and set `BarState::update_text` via the daemon.

use super::BarState;

/// Returns the update notification text, or an empty string if
/// no update is available.
pub fn text(state: &BarState) -> String {
    state.update_text.clone()
}
