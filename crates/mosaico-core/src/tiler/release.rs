//! When the effects a [`Coalescer`](super::Coalescer) holds are applied.
//!
//! Held effects wait for input to go quiet: while the user is still
//! pressing keys, the next press would supersede them anyway.

use std::time::{Duration, Instant};

/// How long the platform has to wait with no input arriving before it
/// applies held effects.
///
/// The wait starts once the platform has handled everything it was
/// given, so the time it spends applying effects, which is long for a
/// workspace switch, never counts as quiet.
pub const QUIET: Duration = Duration::from_millis(25);

/// When effects held while the platform waits for input from
/// `waiting_since` are due, if no input arrives first: once it has
/// waited [`QUIET`].
pub fn release_at(waiting_since: Instant) -> Instant {
    waiting_since + QUIET
}

#[cfg(test)]
#[path = "tests/release_tests.rs"]
mod tests;
