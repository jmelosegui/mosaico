//! What the mouse does in the tiler.

use super::Drag;
use super::live::Live;

/// What the mouse does: whether the pointer follows focus, whether a
/// window dropped on a tile moves there, whether dragging the edge between
/// two tiles resizes them, the drag the tiler follows, if any, and the
/// tiles a drag laid out live.
#[derive(Debug, Default)]
pub(in crate::tiler) struct Mouse {
    /// Whether focus the tiler moves takes the pointer along.
    pub(in crate::tiler) follows_focus: bool,
    /// Whether a tiled window dropped on another tile takes its slot, or
    /// goes before it on another monitor (`[mouse] drag_to_swap`).
    pub(in crate::tiler) drag_to_swap: bool,
    /// Whether the edges of a tiled window the user resizes move the splits
    /// they border (`[mouse] resize_splits`).
    pub(in crate::tiler) resize_splits: bool,
    /// The drag the tiler follows.
    pub(super) drag: Option<Drag>,
    /// The tiles a drag laid out live, until they are back where their
    /// workspace has them (`drag_live.rs`).
    pub(super) live: Option<Live>,
}

impl Mouse {
    /// The mouse with the pointer following focus when `follows_focus`,
    /// windows dropped on tiles moving there when `drag_to_swap`, and edges
    /// dragged moving splits when `resize_splits`.
    pub(in crate::tiler) fn new(
        follows_focus: bool,
        drag_to_swap: bool,
        resize_splits: bool,
    ) -> Self {
        Self {
            follows_focus,
            drag_to_swap,
            resize_splits,
            drag: None,
            live: None,
        }
    }
}
