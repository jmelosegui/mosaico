//! What each motion did, for the log.
//!
//! The log gets one line for each step that gives windows a target, one
//! for each placement that snaps for a reason the log should hear, one
//! when a window falls behind, one as each motion ends, with what its
//! frames found, and one once the last motion of a step has ended, with
//! what all of them found: never one per frame. [`Journal`] keeps the few
//! counts these lines need. A window placed once its transition is over,
//! at the target a display change kept for it or behind what was on its
//! way to it, gets the lines of `journal_lines_placed.rs`.

use std::collections::HashMap;
use std::time::Instant;

use crate::WindowId;

use super::MotionSpec;

#[path = "journal_ending.rs"]
mod ending;
#[path = "journal_frames.rs"]
mod frames;
#[path = "journal_lines.rs"]
mod lines;
#[path = "journal_lines_end.rs"]
mod lines_end;
#[path = "journal_lines_placed.rs"]
mod lines_placed;
#[path = "journal_snap.rs"]
mod snap;
#[path = "journal_steps.rs"]
mod steps;

pub use ending::{Ending, Placed, Stop};
pub use lines::{SnapLine, StepLine, StepSnapLine};
pub use lines_end::{BehindLine, EndLine, EndingLine, StepDoneLine};
pub use lines_placed::{Kept, KeptLine, Landed, LandedLine};

/// What the log is told of the motions in flight.
///
/// The applier tells it as it starts a step, as it gives windows a
/// target, what each frame step found, and how each motion ends. It gives
/// back the lines to log.
#[derive(Debug, Default)]
pub struct Journal {
    motions: HashMap<WindowId, Record>,
    /// What the motions that ended of each step still going did.
    tallies: HashMap<u64, steps::Tally>,
    /// The step the windows given a target from now on belong to.
    step: u64,
    /// The windows that started moving, and those that turned towards a
    /// new target, since the last step line.
    started: Vec<WindowId>,
    turned: Vec<WindowId>,
    /// The duration and easing they were given their targets with.
    spec: Option<MotionSpec>,
    /// The lines of the steps closed as a later step turned the last of
    /// their motions, for the next step line.
    closed: Vec<StepDoneLine>,
}

/// What the journal keeps of one motion.
#[derive(Debug, Clone, Copy)]
struct Record {
    started: Instant,
    /// When its latest target is due.
    due: Instant,
    /// The step that gave it its latest target.
    step: u64,
    /// The frames sent for its window, posted to it or not.
    sent: u32,
    /// How many of them found the window had handled the frame before.
    handled: u32,
    /// How many found it still to handle the frame before.
    pending: u32,
}

impl Journal {
    /// Starts a new step: the windows given a target from now on belong
    /// to it, as they do in [`Motions`](super::Motions).
    pub fn begin_step(&mut self) {
        self.step += 1;
    }

    /// Notes that `window` was given a new target at `now` with `spec`: a
    /// motion that starts, or one that turns. Either way it belongs to the
    /// current step from then on, and a step it leaves with no motion of
    /// its own is closed.
    pub fn moved(&mut self, window: WindowId, now: Instant, spec: MotionSpec) {
        let due = now + spec.duration;
        let step = self.step;
        if let Some(record) = self.motions.get_mut(&window) {
            let left = std::mem::replace(&mut record.step, step);
            record.due = due;
            if !self.started.contains(&window) && !self.turned.contains(&window) {
                self.turned.push(window);
            }
            if left != step {
                self.close(left);
            }
        } else {
            let record = Record {
                started: now,
                due,
                step,
                sent: 0,
                handled: 0,
                pending: 0,
            };
            self.motions.insert(window, record);
            self.started.push(window);
        }
        self.spec = Some(spec);
    }

    /// The lines of the steps closed since the last call, as a later step
    /// turned the last of their motions.
    pub fn closed_steps(&mut self) -> Vec<StepDoneLine> {
        std::mem::take(&mut self.closed)
    }

    /// The line of the windows given a target since the last line, if any
    /// were.
    pub fn step_line(&mut self) -> Option<StepLine> {
        let spec = self.spec.take()?;
        let started = std::mem::take(&mut self.started);
        let turned = std::mem::take(&mut self.turned);
        Some(StepLine {
            started,
            turned,
            spec,
        })
    }
}

#[cfg(test)]
#[path = "tests/journal_tests.rs"]
mod tests;
