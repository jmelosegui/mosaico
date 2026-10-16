//! The frames that move windows to the placements that animate.

use std::time::Instant;

use super::super::Platform;
use super::TilingManager;

impl<P: Platform> TilingManager<P> {
    /// Whether windows are moving to placements that animate. While they
    /// are, the daemon asks for a frame on every display refresh and runs
    /// [`frame`](Self::frame) on each.
    pub fn wants_frames(&self) -> bool {
        self.applier.is_moving()
    }

    /// When a frame is due even if no display frame comes, after `after`,
    /// the last frame step the daemon took: when the first motion is due
    /// to end. `None` while nothing is due. The daemon waits no longer
    /// than this, and then runs [`frame`](Self::frame).
    pub fn frame_due(&self, after: Option<Instant>) -> Option<Instant> {
        self.applier.frame_due(after)
    }

    /// Takes the frame step, which moves the windows that are moving on.
    /// The daemon runs it on a display frame, and once
    /// [`frame_due`](Self::frame_due) has come.
    pub fn frame(&mut self) {
        self.applier.frame(&mut self.platform);
    }

    /// Turns transitions off for the rest of the session, as on a platform
    /// that cannot move windows frame by frame: every window is placed at
    /// once from then on. For a daemon that cannot have display frames.
    pub fn turn_off_transitions(&mut self) {
        self.applier.turn_off_transitions(&mut self.platform);
    }
}
