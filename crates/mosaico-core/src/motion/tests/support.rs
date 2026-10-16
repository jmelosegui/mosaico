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
