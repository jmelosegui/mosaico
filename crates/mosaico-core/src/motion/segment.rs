//! One rect moving from where it is to where it goes.

use std::time::{Duration, Instant};

use crate::Rect;

use super::shape::Shape;
use super::{MotionSpec, Velocity};

/// One rect moving from `from` to `to`, starting at `start` and lasting
/// `duration`.
///
/// A segment is a function of time: it says where the rect is at any
/// instant, asked in any order, from the clock alone. A frame that comes
/// late finds the rect where it should be by then, so dropped frames do
/// not slow a motion down.
///
/// Each edge of the rect moves on its own, in fractions of a pixel, and is
/// rounded to the nearest one, a half always up, so rects that move
/// together along the same curve keep the gaps between them exactly. No
/// edge ever goes back or past where it goes, so the rect stays within the
/// box around `from` and `to`, but for one pixel: a rect is never narrower
/// or shorter than that, so one shrinking to nothing can stick out by it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    from: Rect,
    to: Rect,
    start: Instant,
    duration: Duration,
    shape: Shape,
}

impl Segment {
    /// A rect at rest at `from` that follows the spec's easing to `to`,
    /// for the spec's duration.
    pub fn eased(from: Rect, to: Rect, start: Instant, spec: MotionSpec) -> Self {
        let shape = Shape::Eased(spec.easing);
        Self {
            from,
            to,
            start,
            duration: spec.duration,
            shape,
        }
    }

    /// A rect that leaves `from` with `velocity` and comes to rest at `to`
    /// once `duration` has passed.
    ///
    /// This is how a motion takes a new target halfway: started where the
    /// old segment has the rect, as fast as it moves there, the rect goes
    /// on without a jump in place or speed. So that it never passes the new
    /// target, each edge takes as much of its velocity as it can: one
    /// moving away from its target leaves at rest, and one moving faster
    /// than three times its average speed leaves at that speed.
    pub fn hermite(
        from: Rect,
        velocity: Velocity,
        to: Rect,
        start: Instant,
        duration: Duration,
    ) -> Self {
        let seconds = duration.as_secs_f64();
        let shape = Shape::hermite(edges(from), edges(to), velocity, seconds);
        Self {
            from,
            to,
            start,
            duration,
            shape,
        }
    }

    /// Where the segment ends.
    pub fn target(&self) -> Rect {
        self.to
    }

    /// When the segment ends: its start plus its duration.
    pub fn end(&self) -> Instant {
        self.start + self.duration
    }

    /// Whether the segment has ended by `now`. One that lasts no time has
    /// ended at once.
    pub fn is_done(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.start) >= self.duration
    }

    /// The rect at `now`, in whole pixels: `from` until the segment
    /// starts, exactly `to` from its end on, and never narrower or shorter
    /// than one pixel in between, even on its way to a target that is.
    pub fn rect_at(&self, now: Instant) -> Rect {
        if self.is_done(now) {
            return self.to;
        }
        round(self.position(self.time(now)))
    }

    /// How fast the rect's edges move at `now`.
    ///
    /// Zero from the segment's end on. An eased segment is at rest at its
    /// start too, before it has moved; a Hermite one leaves with the
    /// velocity it took.
    pub fn velocity_at(&self, now: Instant) -> Velocity {
        if self.is_done(now) {
            return Velocity::ZERO;
        }
        let seconds = self.duration.as_secs_f64();
        let (from, to) = (edges(self.from), edges(self.to));
        self.shape.velocity(from, to, self.time(now), seconds)
    }

    /// The fraction of the duration that has passed at `now`, while the
    /// segment has not ended.
    fn time(&self, now: Instant) -> f64 {
        let elapsed = now.saturating_duration_since(self.start);
        elapsed.as_secs_f64() / self.duration.as_secs_f64()
    }

    /// The rect's edges, in fractions of a pixel, once `s` of the duration
    /// has passed.
    fn position(&self, s: f64) -> [f64; 4] {
        let seconds = self.duration.as_secs_f64();
        let (from, to) = (edges(self.from), edges(self.to));
        self.shape.position(from, to, s, seconds)
    }
}

/// The edges of a rect, `[left, top, right, bottom]`.
fn edges(rect: Rect) -> [f64; 4] {
    let [x, y, width, height] = [rect.x, rect.y, rect.width, rect.height].map(f64::from);
    [x, y, x + width, y + height]
}

/// The rect whose edges are nearest to these, at least one pixel wide
/// and tall. A half rounds up on either side of 0, so moving the edges by
/// whole pixels moves the rect by as many.
fn round(edges: [f64; 4]) -> Rect {
    let [left, top, right, bottom] = edges.map(|edge| (edge + 0.5).floor());
    let (width, height) = ((right - left).max(1.0), (bottom - top).max(1.0));
    // The casts saturate: a rect far out of range stays at the edge of the
    // coordinates instead of wrapping around.
    Rect::new(left as i32, top as i32, width as i32, height as i32)
}

#[cfg(test)]
#[path = "tests/segment_tests.rs"]
mod tests;
