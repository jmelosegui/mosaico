//! How long the daemon loop may wait for messages.

use std::time::{Duration, Instant};

use crate::tiler::release_at;

use super::Scheduler;

impl Scheduler {
    /// Starts a wait at `now`, once the loop has run every step it was
    /// given, unless the last wait goes on, and returns how long it may
    /// wait for messages, or `None` when nothing is due until one
    /// arrives. `held_since` is when the effects the tiling manager holds
    /// started being held, if any, and `frame_due` the first frame step due
    /// after the [`last_frame_step`](Self::last_frame_step), as at the end of
    /// a motion: the wait ends by then with it. One at or before it is let go.
    /// `wheel_due` is when the wheel's turns waiting on a bar are due, as
    /// [`BarManager::wheel_due`](crate::bar::BarManager::wheel_due) says:
    /// the wait ends by then with the wheel step.
    pub fn begin_wait(
        &mut self,
        held_since: Option<Instant>,
        frame_due: Option<Instant>,
        wheel_due: Option<Instant>,
        now: Instant,
    ) -> Option<Duration> {
        let waiting_since = *self.waiting_since.get_or_insert(now);
        self.hold = held_since.map(|since| match self.hold {
            Some((seen, first_wait)) if seen == since => (since, first_wait),
            _ => (since, now),
        });
        self.release = self
            .hold
            .map(|(_, first_wait)| release_at(first_wait, waiting_since));
        self.frame_due = self.not_stepped(frame_due);
        self.wheel_due = wheel_due;
        let due = if self.settle.is_due(now) {
            Some(Duration::ZERO)
        } else {
            match self.release {
                Some(release) => Some(release.saturating_duration_since(now)),
                None => self.settle.remaining(now),
            }
        };
        let frame = self.frame_due.map(|due| due.saturating_duration_since(now));
        let wheel = wheel_due.map(|due| due.saturating_duration_since(now));
        due.into_iter().chain(frame).chain(wheel).min()
    }
}
