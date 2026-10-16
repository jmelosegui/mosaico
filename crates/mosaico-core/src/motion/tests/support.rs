//! Shorthands for the motion tests.

use std::time::Duration;

use super::{Easing, MotionSpec};

/// `millis` milliseconds.
pub(super) fn ms(millis: u64) -> Duration {
    Duration::from_millis(millis)
}

/// A motion lasting `millis` milliseconds along `easing`.
pub(super) fn spec(millis: u64, easing: Easing) -> MotionSpec {
    MotionSpec {
        duration: ms(millis),
        easing,
    }
}

/// The windows `tick` has frames for, each with the rect of its frame.
pub(super) fn flights(tick: &super::Tick) -> Vec<(crate::WindowId, crate::Rect)> {
    tick.in_flight.iter().map(|f| (f.window, f.rect)).collect()
}
