//! What the table of moving windows says about them.

use std::time::Instant;

use crate::{Rect, WindowId};

use super::{Motion, Motions};

impl Motions {
    /// Where the window is moving to, if it is moving.
    pub fn target(&self, window: WindowId) -> Option<Rect> {
        self.motion(window).map(|m| m.segment.target())
    }

    /// Where the window's motion has it at `now`, if it is moving.
    pub fn rect_at(&self, window: WindowId, now: Instant) -> Option<Rect> {
        self.motion(window).map(|m| m.segment.rect_at(now))
    }

    /// Where the window is at `now`, as far as the table knows, if it is
    /// moving: where its motion has it, unless the last frame found it
    /// still to handle the one before, and so where that one found it.
    pub fn position(&self, window: WindowId, now: Instant) -> Option<Rect> {
        let motion = self.motion(window)?;
        Some(if motion.pending > 0 {
            motion.reached
        } else {
            motion.segment.rect_at(now)
        })
    }

    /// Whether the window is moving.
    pub fn is_moving(&self, window: WindowId) -> bool {
        self.motion(window).is_some()
    }

    /// Whether no window is moving, so no more frames are needed.
    pub fn is_empty(&self) -> bool {
        self.moving.is_empty()
    }

    /// When the first motion to end is due to, or `None` when no window
    /// is moving: a frame is due by then, even if no display frame comes.
    /// A motion leaves the table on the first frame at or after its end,
    /// so no end that a frame was taken at or after is left in it.
    pub fn deadline(&self) -> Option<Instant> {
        self.moving.iter().map(|m| m.segment.end()).min()
    }

    fn motion(&self, window: WindowId) -> Option<&Motion> {
        self.moving.iter().find(|m| m.window == window)
    }
}
