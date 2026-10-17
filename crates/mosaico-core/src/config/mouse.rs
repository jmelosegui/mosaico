//! The `[mouse]` section of `config.toml`: the pointer and focus, and
//! dragging windows.

use serde::{Deserialize, Serialize};

/// Mouse integration settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MouseConfig {
    /// Move the cursor to the center of the focused window on
    /// keyboard navigation.
    pub follows_focus: bool,
    /// Automatically focus the window under the cursor without clicking.
    pub focus_follows_mouse: bool,
    /// A tiled window dropped on another tile of its monitor swaps places
    /// with it, and one dropped on another monitor joins its workspace
    /// before the tile under the pointer. Off, a dropped window goes back
    /// to its slot, or joins the end of the other monitor's workspace.
    pub drag_to_swap: bool,
    /// Dragging the edge between two tiles moves the split between them,
    /// and a corner where two splits meet moves both. Off, a tiled window
    /// the user resizes goes back to its slot.
    pub resize_splits: bool,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            follows_focus: true,
            focus_follows_mouse: false,
            drag_to_swap: true,
            resize_splits: true,
        }
    }
}
