//! The time source commands poll with.

use std::time::{Duration, Instant};

/// The time source commands poll with.
pub trait Clock {
    /// The current time.
    fn now(&self) -> Instant;

    /// Waits for `duration`.
    fn sleep(&self, duration: Duration);
}

/// The real clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}
