//! The windows that are moving, each to its target.

use std::time::Instant;

use crate::{Rect, WindowId};

use super::{MotionSpec, Segment, Velocity};

#[path = "motions_frame.rs"]
mod frame;
#[path = "motions_query.rs"]
mod query;

pub use frame::{MAX_LAG, Tick};

/// The table of moving windows, each along a [`Segment`] to its target.
///
/// One table drives every moving window from the same clock: the windows
/// a step moves start together, and each frame puts all of them at the
/// same point of their curves. Each application handles its frames when it
/// gets to them, so keeping neighbours aligned is best effort: when one
/// window of a step falls behind, every motion of the step ends at once
/// (see [`MAX_LAG`]).
///
/// While it is not [empty](Self::is_empty), the daemon takes a
/// [`frame`](Self::frame) on every display frame, posts its frames to the
/// platform, and hands back what the platform found of each window with
/// [`take`](Self::take). A motion ends at its end time, or when a window of
/// its step falls behind: the caller then places the window at its target
/// in full, which the platform orders behind the frames posted before.
#[derive(Debug, Default)]
pub struct Motions {
    /// In the order their motions started.
    moving: Vec<Motion>,
    /// The step the motions given a target from now on belong to.
    step: u64,
}

/// One moving window.
#[derive(Debug, Clone, Copy)]
struct Motion {
    window: WindowId,
    segment: Segment,
    /// The rect the window was last sent, or where it was when its motion
    /// started, which a frame does not send again.
    sent: Rect,
    /// The step that last gave the window a target.
    step: u64,
    /// Whether the window has been sent a frame of this motion.
    framed: bool,
    /// How many frames in a row found the window still to handle its last.
    pending: u32,
    /// Where a frame last found the window, or where it was when its
    /// motion started.
    reached: Rect,
    /// When a frame last found the window somewhere new, or when it handled
    /// its first frame, and until then when its motion started.
    moved_at: Instant,
    /// What the last frame posted to the window: `None` before its first,
    /// and when it found the window still to handle the one before.
    posted: Option<Rect>,
}

impl Motions {
    /// Starts a new step: the windows given a target from now on move
    /// together, and end together when one of them falls behind.
    pub fn begin_step(&mut self) {
        self.step += 1;
    }

    /// Sends `window` to `to`, from `now`, as part of the current step.
    ///
    /// - A window already moving to `to` goes on as it is, with the spec
    ///   it started with: repeating a target changes nothing.
    /// - A window moving somewhere else turns towards `to` from where it
    ///   is, keeping as much of its speed as does not carry it past `to`,
    ///   and comes to rest there once the spec's full duration has passed
    ///   (see [`Segment::hermite`]). Its motion goes on, so its frames are
    ///   not the first of a motion.
    /// - A window that is not moving eases from `from`, where it is, along
    ///   the spec's easing. So does a moving one that would keep none of
    ///   its speed: one at rest, as before its motion has moved at all, or
    ///   one moving straight away from `to`. It eases from where its motion
    ///   has it, as fast as a new motion starts; `from` is only for windows
    ///   that are not moving.
    pub fn retarget(
        &mut self,
        window: WindowId,
        from: Rect,
        to: Rect,
        now: Instant,
        spec: MotionSpec,
    ) {
        let step = self.step;
        let Some(motion) = self.moving.iter_mut().find(|m| m.window == window) else {
            self.moving.push(Motion {
                window,
                segment: Segment::eased(from, to, now, spec),
                sent: from,
                step,
                framed: false,
                pending: 0,
                reached: from,
                moved_at: now,
                posted: None,
            });
            return;
        };
        if motion.segment.target() == to {
            return;
        }
        let rect = motion.segment.rect_at(now);
        let velocity = motion.segment.velocity_at(now);
        let turn = Segment::hermite(rect, velocity, to, now, spec.duration);
        // The turn keeps the speed it can: none means a start from rest.
        motion.segment = if turn.velocity_at(now) == Velocity::ZERO {
            Segment::eased(rect, to, now, spec)
        } else {
            turn
        };
        motion.step = step;
    }

    /// Ends the window's motion, wherever the window is, as when it is
    /// destroyed, hidden or grabbed by the user, or placed elsewhere.
    /// Returns the target it was moving to, or `None` when it was not
    /// moving.
    pub fn end(&mut self, window: WindowId) -> Option<Rect> {
        let index = self.moving.iter().position(|m| m.window == window)?;
        Some(self.moving.remove(index).segment.target())
    }

    /// Ends every motion and returns each window with the target it was
    /// moving to, in the order the motions started.
    ///
    /// The daemon stopping and transitions turning off finish every motion
    /// at its target: the caller places each window there.
    pub fn clear(&mut self) -> Vec<(WindowId, Rect)> {
        let moving = self.moving.drain(..);
        moving.map(|m| (m.window, m.segment.target())).collect()
    }
}

#[cfg(test)]
#[path = "tests/motions_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/retarget_tests.rs"]
mod retarget_tests;
