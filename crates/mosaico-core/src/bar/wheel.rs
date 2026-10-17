//! Turns of the wheel over a bar, added up until the wheel goes quiet.

use std::time::{Duration, Instant};

/// How long the wheel has to stay still before the turns it made are
/// applied.
///
/// A flick of the wheel reports several notches a few milliseconds
/// apart, and a touchpad reports a stream of fractions of a notch for as
/// long as the fingers move. Both add up, and switch once, as many
/// workspaces on as they add up to. Notches turned one at a time, with a
/// pause between them, switch once each.
pub const WHEEL_QUIET: Duration = Duration::from_millis(120);

/// How long a fraction of a notch left over once turns are applied is
/// kept for the turns that follow. A high-resolution wheel turned slowly
/// reports a fraction at a time, each after the wheel went quiet.
const KEPT: Duration = Duration::from_secs(1);

/// How close to a whole number of notches the turns must add up to for
/// it to count, since fractions such as tenths do not add up exactly.
const SLACK: f64 = 1e-3;

/// The turns of the wheel over one bar: the ones waiting for the wheel to
/// go quiet, and the fraction of a notch left over from the ones applied.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct WheelSteps {
    /// The notches turned and not applied yet, down or right positive.
    notches: f64,
    /// When the last turn came.
    last: Option<Instant>,
    /// Whether turns are waiting to be applied.
    waiting: bool,
}

impl WheelSteps {
    /// Adds a turn of `notches` that came at `now`. A turn that is not a
    /// number is left out. A fraction of a notch left over from the turns
    /// applied adds up with it, unless the turn goes the other way or
    /// comes a second or more after the last one.
    pub fn turn(&mut self, notches: f32, now: Instant) {
        if !notches.is_finite() {
            return;
        }
        let notches = f64::from(notches);
        let idle = self
            .last
            .is_none_or(|last| now.saturating_duration_since(last) >= KEPT);
        if !self.waiting && (idle || notches * self.notches < 0.0) {
            self.notches = 0.0;
        }
        self.notches += notches;
        self.last = Some(now);
        self.waiting = true;
    }

    /// When the turns waiting are due, [`WHEEL_QUIET`] after the last one,
    /// or `None` when none are.
    pub fn due(&self) -> Option<Instant> {
        self.last
            .filter(|_| self.waiting)
            .map(|last| last + WHEEL_QUIET)
    }

    /// The whole steps the turns waiting add up to, once they are due at
    /// `now`: positive down or right, negative up or left. A fraction of a
    /// notch left over is kept for the turns that follow (see
    /// [`turn`](Self::turn)). `None` while the wheel is still turning, or
    /// when no turns are waiting.
    pub fn take(&mut self, now: Instant) -> Option<i32> {
        if self.due().is_none_or(|due| now < due) {
            return None;
        }
        let steps = (self.notches + self.notches.signum() * SLACK).trunc();
        self.notches -= steps;
        if self.notches.abs() < SLACK {
            self.notches = 0.0;
        }
        self.waiting = false;
        Some(steps as i32)
    }

    /// Forgets every turn: the ones waiting and what was left over.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
#[path = "tests/wheel_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/wheel_rest_tests.rs"]
mod rest_tests;
