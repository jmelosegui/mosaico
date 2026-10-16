//! When the effects a [`Coalescer`](super::Coalescer) holds are applied.
//!
//! Held effects wait for input to go quiet: while the user is still
//! pressing keys, the next press would supersede them anyway. They never
//! wait longer than [`MAX_HOLD`], because some input never goes quiet.

use std::time::{Duration, Instant};

/// How long the daemon has to wait with no input arriving before it
/// has the tiling manager apply the effects it holds.
///
/// The wait starts once the daemon has handled everything it was
/// given, so the time spent applying effects, which is long for a
/// workspace switch, never counts as quiet.
pub const QUIET: Duration = Duration::from_millis(25);

/// The longest effects are held while input keeps arriving.
///
/// A title that keeps changing never lets the daemon wait [`QUIET`], and
/// would otherwise hold focus changes and moves back for as long as it
/// lasts. A window that reports its position on every frame of an
/// animation does not count: position reports are not input to the
/// [`Scheduler`](crate::daemon::Scheduler), whoever moved the window.
///
/// The cap counts from the first time the daemon waits with the
/// effects held, not from when they were held, so the time spent
/// handling a slow batch or starting up never shortens the quiet window.
/// Only input that keeps ending every wait early reaches it.
///
/// 100 ms is four times [`QUIET`], so an ordinary burst of key presses
/// still ends on the quiet window. It keeps the delay a stream can add at
/// about the point where a lag starts to be noticed. Presses that queue
/// up while a slow focus change is being applied are still coalesced,
/// since they arrive together in the next batch. A longer hold would
/// coalesce more under a stream, but moves would visibly lag.
pub const MAX_HOLD: Duration = Duration::from_millis(100);

/// When held effects are due, if the daemon waits for input from
/// `waiting_since` and none arrives first: once it has waited [`QUIET`],
/// and no later than [`MAX_HOLD`] after `first_wait`, the first time it
/// waited with these effects held.
pub fn release_at(first_wait: Instant, waiting_since: Instant) -> Instant {
    (waiting_since + QUIET).min(first_wait + MAX_HOLD)
}

#[cfg(test)]
#[path = "tests/release_tests.rs"]
mod tests;
