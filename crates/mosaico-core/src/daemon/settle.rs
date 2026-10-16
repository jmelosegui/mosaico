//! Coalesces bursts of display change notifications.
//!
//! Connecting or disconnecting a monitor, and especially switching a KVM,
//! produces several display change notifications over a second or so,
//! and the monitor list read in between can be a transient one.
//! Rebuilding on each of them retiles every window several times and can
//! briefly park a monitor that is only renegotiating its signal. The
//! daemon instead waits until no change has arrived for [`SETTLE`] and
//! rebuilds once against the final configuration.

use std::time::{Duration, Instant};

/// How long the display configuration must stay quiet before it is
/// applied.
pub const SETTLE: Duration = Duration::from_millis(750);

/// Pending display change, applied once it has been quiet for [`SETTLE`].
#[derive(Debug, Default)]
pub(super) struct DisplaySettle {
    due: Option<Instant>,
}

impl DisplaySettle {
    /// Records a display change notification received at `now`, pushing
    /// the deadline back if one is already pending.
    pub(super) fn note(&mut self, now: Instant) {
        self.due = Some(now + SETTLE);
    }

    /// Whether a pending change is ready to apply at `now`.
    pub(super) fn is_due(&self, now: Instant) -> bool {
        self.due.is_some_and(|due| now >= due)
    }

    /// Clears and reports a change that is ready to apply at `now`.
    pub(super) fn take_due(&mut self, now: Instant) -> bool {
        let due = self.is_due(now);
        if due {
            self.due = None;
        }
        due
    }

    /// Time left until the pending change is due, if one is pending.
    pub(super) fn remaining(&self, now: Instant) -> Option<Duration> {
        self.due.map(|due| due.saturating_duration_since(now))
    }
}

#[cfg(test)]
#[path = "tests/settle_tests.rs"]
mod tests;
