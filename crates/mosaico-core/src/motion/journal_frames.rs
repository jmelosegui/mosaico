//! What the frames of a frame step tell the journal.

use std::time::Instant;

use crate::WindowId;
use crate::frames::{Frame, FrameOutcome};

use super::{BehindLine, Ending, EndingLine, Journal, Stop};

impl Journal {
    /// Counts the frames of a frame step at `now`, with `outcomes`, what
    /// the platform found of the window of each, in order. The motion of a
    /// window found maximized, minimized or gone ends there, and its lines
    /// are given back. Frames for windows it has no motion of are left
    /// out.
    pub fn framed(
        &mut self,
        frames: &[Frame],
        outcomes: &[FrameOutcome],
        now: Instant,
    ) -> Vec<EndingLine> {
        let mut lines = Vec::new();
        for (frame, &outcome) in frames.iter().zip(outcomes) {
            let window = frame.window;
            let Some(record) = self.motions.get_mut(&window) else {
                continue;
            };
            record.sent += 1;
            let ending = match outcome {
                FrameOutcome::Handled { .. } => {
                    record.handled += 1;
                    continue;
                }
                FrameOutcome::Pending => {
                    record.pending += 1;
                    continue;
                }
                FrameOutcome::Maximized => Ending::Stopped(Stop::Maximized),
                FrameOutcome::Minimized => Ending::Stopped(Stop::Minimized),
                FrameOutcome::Gone => Ending::Destroyed,
            };
            lines.extend(self.ended(window, ending, None, now));
        }
        lines
    }

    /// The line of `window`, which a frame step found had fallen behind,
    /// as [`Motions::take`](super::super::Motions::take) decides: at
    /// which frame of its motion. `None` when it has no motion of the
    /// window.
    pub fn fell_behind(&self, window: WindowId) -> Option<BehindLine> {
        let record = self.motions.get(&window)?;
        Some(BehindLine {
            window,
            frame: record.sent,
        })
    }
}

#[cfg(test)]
#[path = "tests/journal_frames_tests.rs"]
mod tests;
