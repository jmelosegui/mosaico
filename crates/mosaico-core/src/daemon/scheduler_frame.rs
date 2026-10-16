//! What the scheduler keeps about its frame steps, and tells the loop.

use std::time::Instant;

use super::Scheduler;

impl Scheduler {
    /// When the last frame step was taken, if one was. An end of a motion
    /// at or before it has had its step, and
    /// [`begin_wait`](Scheduler::begin_wait) lets such an end go, so the
    /// loop gives `begin_wait` the earliest end after the last step: given
    /// one that had its step, the wait would not end for the ends after it.
    pub fn last_frame_step(&self) -> Option<Instant> {
        self.frame_stepped
    }

    /// `due`, unless a frame step was taken at or after it, which it
    /// needs no other for.
    pub(super) fn not_stepped(&self, due: Option<Instant>) -> Option<Instant> {
        let stepped = self.frame_stepped;
        due.filter(|&due| stepped.is_none_or(|stepped| due > stepped))
    }
}
