//! What each display frame does to the moving windows, and what the
//! platform found of them.

use std::time::{Duration, Instant};

use crate::frames::{Frame, FrameOutcome};
use crate::{Rect, WindowId};

use super::Motions;

/// How long a window found still to handle the frame before may go
/// without being seen to move, before it has fallen behind and every
/// motion of its step ends at once.
///
/// A window whose application is slow stays where it is while its
/// neighbours move on, into the space it still takes up. Ending the whole
/// step at once puts every window in its place instead. An application
/// slower than this would take most of a transition to catch up. It is a
/// time, not a number of frames, so it is the same at any refresh rate.
pub const MAX_LAG: Duration = Duration::from_millis(50);

/// What one display frame does to the moving windows.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tick {
    /// The frames to post, one for each moving window whose rect is not
    /// the one it was last sent, in the order their motions started. Only
    /// the first frame of a motion is marked so.
    pub in_flight: Vec<Frame>,
    /// The motions that are over, each with its window's exact target, to
    /// be placed there in full: those whose end time has come, then those
    /// of a step one of whose windows fell behind, each in the order their
    /// motions started. They have left the table.
    pub finished: Vec<(WindowId, Rect)>,
    /// The windows on their way that a frame found somewhere new, for what
    /// follows the window, such as its border, to follow it: each where it
    /// was found, or, if it kept up, where this frame takes it (see
    /// [`Motions::take`]).
    pub reached: Vec<(WindowId, Rect)>,
    /// The windows whose motions a frame found over without a placement:
    /// maximized or minimized on their way, which stops them where they
    /// are, or gone. They have left the table.
    pub stopped: Vec<WindowId>,
    /// The windows that fell behind, in the order of their frames: every
    /// motion of their steps is in `finished`.
    pub behind: Vec<WindowId>,
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
                let first = !std::mem::replace(&mut motion.framed, true);
                let window = motion.window;
                tick.in_flight.push(Frame {
                    window,
                    rect,
                    first,
                });
            }
            true
        });
        tick
    }

    /// Takes what the platform found of each window `tick` has a frame
    /// for, one outcome per frame, in order, and adds to `tick` what
    /// follows.
    ///
    /// - A window that handled the frame before goes on, and one found
    ///   somewhere new is in [`Tick::reached`]: where it was found, or,
    ///   when it took the frame posted at the frame before exactly, where
    ///   this frame takes it. So what follows a window that keeps up does
    ///   not trail it, and what follows a slow one does not run ahead.
    /// - A window found still to handle it, and not seen to move for
    ///   [`MAX_LAG`] since its first frame was handled, has fallen behind,
    ///   and is in [`Tick::behind`]: every motion of its step ends at once,
    ///   in [`Tick::finished`].
    /// - A window maximized or minimized meanwhile stops where it is, and
    ///   nothing forces it back. One that is gone is forgotten. Both are in
    ///   [`Tick::stopped`].
    pub fn take(&mut self, tick: &mut Tick, outcomes: &[FrameOutcome], now: Instant) {
        let mut behind = Vec::new();
        for (frame, &outcome) in tick.in_flight.iter().zip(outcomes) {
            let Some(index) = self.moving.iter().position(|m| m.window == frame.window) else {
                continue;
            };
            let motion = &mut self.moving[index];
            match outcome {
                FrameOutcome::Handled { reached } => {
                    motion.pending = 0;
                    if frame.first {
                        motion.moved_at = now;
                    }
                    if reached != motion.reached {
                        let kept_up = motion.posted == Some(reached);
                        let at = if kept_up { frame.rect } else { reached };
                        (motion.reached, motion.moved_at) = (reached, now);
                        tick.reached.push((frame.window, at));
                    }
                    motion.posted = Some(frame.rect);
                }
                FrameOutcome::Pending => {
                    motion.pending = motion.pending.saturating_add(1);
                    motion.posted = None;
                    if now.saturating_duration_since(motion.moved_at) >= MAX_LAG {
                        behind.push(motion.step);
                        tick.behind.push(frame.window);
                    }
                }
                FrameOutcome::Maximized | FrameOutcome::Minimized | FrameOutcome::Gone => {
                    self.moving.remove(index);
                    tick.stopped.push(frame.window);
                }
            }
        }
        self.moving.retain(|motion| {
            let ends = behind.contains(&motion.step);
            if ends {
                tick.finished.push((motion.window, motion.segment.target()));
            }
            !ends
        });
    }
}

#[cfg(test)]
#[path = "tests/frame_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/behind_tests.rs"]
mod behind_tests;

#[cfg(test)]
#[path = "tests/lag_tests.rs"]
mod lag_tests;
