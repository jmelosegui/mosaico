//! How a placement reaches the screen: when, and whether its window
//! moves there over time.

/// When a [`Effect::Place`](super::Effect::Place) has to reach the
/// screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timing {
    /// Apply it right away.
    Now,
    /// The tiling manager keeps only the latest position per window and
    /// applies it once input goes quiet. Used for moves the user tends
    /// to repeat quickly, so a burst of key presses repaints once.
    ///
    /// Effects that follow for the same window
    /// ([`Effect::Border`](super::Effect::Border),
    /// [`Effect::CenterCursor`](super::Effect::CenterCursor)) must wait
    /// for the move.
    Coalesced,
}

/// Whether an [`Effect::Place`](super::Effect::Place) puts its window in
/// place at once, or may move it there over a short time.
///
/// The layout passes have every placement snap. As the tiler finishes a
/// step, it marks the placements that may animate, with
/// [`step_animates`](crate::motion::step_animates), only while
/// transitions are on: with them off, every placement snaps, as it did
/// before transitions. A placement marked to animate still snaps when it
/// is applied unless
/// [`still_animates`](crate::motion::still_animates) says it may.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    /// Put the window in place at once.
    Snap,
    /// The window may move to its place over a short time.
    Animate,
}

#[cfg(test)]
impl super::Effect {
    /// A placement of `window` at `rect` that snaps, as the layout passes
    /// emit it.
    pub(crate) fn snap(window: crate::WindowId, rect: crate::Rect, timing: Timing) -> Self {
        Self::Place {
            window,
            rect,
            timing,
            transition: Transition::Snap,
        }
    }
}
