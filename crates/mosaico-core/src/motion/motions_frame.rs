//! What each display frame does to the moving windows.

use std::time::Instant;

use crate::{Rect, WindowId};

use super::Motions;

/// What one display frame does to the moving windows.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tick {
    /// The windows to move on this frame, each to the rect it has now, in
    /// the order their motions started. A window whose rect is the one it
    /// was last sent is left out.
    pub in_flight: Vec<(WindowId, Rect)>,
    /// The motions that ended by this frame, each with the window's exact
    /// target, in the same order. They have left the table.
    pub finished: Vec<(WindowId, Rect)>,
}

impl Motions {
    /// Works out where the moving windows are at `now`.
    ///
    /// Every frame is worked out from `now` alone, so frames may come at
    /// any times and any number of them may be missed: a late frame puts
    /// the windows where they should be by then. A motion ends at its end
    /// time with its exact target, whatever frames it got before.
    pub fn frame(&mut self, now: Instant) -> Tick {
        let mut tick = Tick::default();
        self.moving.retain_mut(|motion| {
            if motion.segment.is_done(now) {
                tick.finished.push((motion.window, motion.segment.target()));
                return false;
            }
            let rect = motion.segment.rect_at(now);
            if motion.sent != rect {
                motion.sent = rect;
                tick.in_flight.push((motion.window, rect));
            }
            true
        });
        tick
    }
}

#[cfg(test)]
#[path = "tests/frame_tests.rs"]
mod tests;
