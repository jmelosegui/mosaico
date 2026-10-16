//! The lines of a motion as it ends, and of its step once the last of its
//! motions has: what all of them did, all told.

use std::time::Instant;

use crate::WindowId;

use super::{EndLine, Ending, EndingLine, Journal, Placed, Record, StepDoneLine};

/// What the motions of one step that have ended did, so far.
#[derive(Debug, Default)]
pub(super) struct Tally {
    /// Their windows, in the order their motions ended.
    windows: Vec<WindowId>,
    sent: u32,
    handled: u32,
    pending: u32,
    /// The window found pending the most, and how often, if any was.
    slowest: Option<(WindowId, u32)>,
}

impl Journal {
    /// The lines of the motion of `window`, which ends at `now` for
    /// `ending`, with how the placement that ended it was `placed`, if one
    /// did: its own line, and its step's once it was the last of its step
    /// to end. Nothing when the journal has no motion of the window.
    pub fn ended(
        &mut self,
        window: WindowId,
        ending: Ending,
        placed: Option<Placed>,
        now: Instant,
    ) -> Vec<EndingLine> {
        let Some(record) = self.motions.remove(&window) else {
            return Vec::new();
        };
        let line = EndLine {
            window,
            ending,
            took: now.saturating_duration_since(record.started),
            planned: record.due.saturating_duration_since(record.started),
            sent: record.sent,
            handled: record.handled,
            pending: record.pending,
            placed,
        };
        let mut lines = vec![EndingLine::Motion(line)];
        lines.extend(self.tally(window, &record).map(EndingLine::Step));
        lines
    }

    /// Adds the motion of `window`, which just ended with `record`, to
    /// what its step did, and gives the step's line once no other motion
    /// of the step is left. A window that a later step gave a new target
    /// counts in that step.
    fn tally(&mut self, window: WindowId, record: &Record) -> Option<StepDoneLine> {
        let tally = self.tallies.entry(record.step).or_default();
        tally.windows.push(window);
        tally.sent += record.sent;
        tally.handled += record.handled;
        tally.pending += record.pending;
        let slower = tally.slowest.is_none_or(|(_, most)| record.pending > most);
        if record.pending > 0 && slower {
            tally.slowest = Some((window, record.pending));
        }
        self.done(record.step)
    }

    /// Closes `step`, which a later step took a motion from, if no motion
    /// of it is left: its line, if any of its motions ended, is given with
    /// the next step line, and what it kept is dropped.
    pub(super) fn close(&mut self, step: u64) {
        let line = self.done(step);
        self.closed.extend(line);
    }

    /// The line of `step`, once no motion of it is left and one of them
    /// ended, which drops what it kept.
    fn done(&mut self, step: u64) -> Option<StepDoneLine> {
        if self.motions.values().any(|other| other.step == step) {
            return None;
        }
        let tally = self.tallies.remove(&step)?;
        Some(StepDoneLine {
            windows: tally.windows,
            sent: tally.sent,
            handled: tally.handled,
            pending: tally.pending,
            slowest: tally.slowest,
        })
    }
}

#[cfg(test)]
#[path = "tests/journal_steps_tests.rs"]
mod tests;
