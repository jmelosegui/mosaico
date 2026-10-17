//! What the mouse does in the tiler.

use super::Drag;

/// What the mouse does: whether the pointer follows focus, and the drag
/// the tiler follows, if any.
#[derive(Debug, Default)]
pub(in crate::tiler) struct Mouse {
    /// Whether focus the tiler moves takes the pointer along.
    pub(in crate::tiler) follows_focus: bool,
    /// The drag the tiler follows.
    pub(super) drag: Option<Drag>,
}

impl Mouse {
    /// The mouse with the pointer following focus when `follows_focus`.
    pub(in crate::tiler) fn new(follows_focus: bool) -> Self {
        Self {
            follows_focus,
            drag: None,
        }
    }
}
