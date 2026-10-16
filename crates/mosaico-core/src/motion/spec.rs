//! How long a motion lasts and the curve it follows.

use std::time::Duration;

use super::Easing;

/// How long a motion lasts and the curve it follows, as configured when
/// the motion starts.
///
/// A config reload never changes a motion in flight: it goes on with the
/// duration and easing it started with. A new target given after the
/// reload starts its segment with the spec that comes with it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionSpec {
    /// How long the motion lasts. A zero duration ends it at once.
    pub duration: Duration,
    /// The curve a motion that starts at rest follows. One that continues
    /// another motion comes to rest along a curve of its own instead, see
    /// [`Segment::hermite`](super::Segment::hermite).
    pub easing: Easing,
}
