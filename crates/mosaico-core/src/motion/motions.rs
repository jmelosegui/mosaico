//! The windows that are moving, each to its target.

use std::time::Instant;

use crate::{Rect, WindowId};

use super::{MotionSpec, Segment, Velocity};

#[path = "motions_frame.rs"]
mod frame;

pub use frame::Tick;

/// The table of moving windows, each along a [`Segment`] to its target.
///
/// One table drives every moving window from the same clock: the windows
/// a step moves start together, and each frame puts all of them at the
/// same point of their curves, so neighbours stay aligned.
///
/// While it is not [empty](Self::is_empty), the daemon takes a
/// [`frame`](Self::frame) on every display frame and hands its rects to
/// the platform.
#[derive(Debug, Default)]
pub struct Motions {
    /// In the order their motions started.
    moving: Vec<Motion>,
}

/// One moving window.
#[derive(Debug, Clone, Copy)]
struct Motion {
    window: WindowId,
    segment: Segment,
    /// The rect the window was last sent, or where it was when its motion
    /// started, which a frame does not send again.
    sent: Rect,
}

impl Motions {
    /// Sends `window` to `to`, from `now`.
    ///
    /// - A window already moving to `to` goes on as it is, with the spec
    ///   it started with: repeating a target changes nothing.
    /// - A window moving somewhere else turns towards `to` from where it
    ///   is, keeping as much of its speed as does not carry it past `to`,
    ///   and comes to rest there once the spec's full duration has passed
    ///   (see [`Segment::hermite`]).
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
        let Some(motion) = self.moving.iter_mut().find(|m| m.window == window) else {
            self.moving.push(Motion {
                window,
                segment: Segment::eased(from, to, now, spec),
                sent: from,
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
    }

    /// Stops the window's motion, wherever the window is, as when it is
    /// destroyed, hidden or grabbed by the user. Returns the target it was
    /// moving to, or `None` when it was not moving.
    pub fn end(&mut self, window: WindowId) -> Option<Rect> {
        let index = self.moving.iter().position(|m| m.window == window)?;
        Some(self.moving.remove(index).segment.target())
    }

    /// Stops every motion and returns each window with the target it was
    /// moving to, in the order the motions started.
    ///
    /// A display change and the daemon stopping finish every motion at its
    /// target: the caller places each window there.
    pub fn clear(&mut self) -> Vec<(WindowId, Rect)> {
        let moving = self.moving.drain(..);
        moving.map(|m| (m.window, m.segment.target())).collect()
    }

    /// Where the window is moving to, if it is moving.
    pub fn target(&self, window: WindowId) -> Option<Rect> {
        self.motion(window).map(|m| m.segment.target())
    }

    /// Where the window's motion has it at `now`, if it is moving.
    pub fn rect_at(&self, window: WindowId, now: Instant) -> Option<Rect> {
        self.motion(window).map(|m| m.segment.rect_at(now))
    }

    /// Whether the window is moving.
    pub fn is_moving(&self, window: WindowId) -> bool {
        self.motion(window).is_some()
    }

    /// Whether no window is moving, so no more frames are needed.
    pub fn is_empty(&self) -> bool {
        self.moving.is_empty()
    }

    fn motion(&self, window: WindowId) -> Option<&Motion> {
        self.moving.iter().find(|m| m.window == window)
    }
}

#[cfg(test)]
#[path = "tests/motions_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/retarget_tests.rs"]
mod retarget_tests;
