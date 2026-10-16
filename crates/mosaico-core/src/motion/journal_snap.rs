//! Which placements that snap the log hears of, and why.
//!
//! Every reason comes from the decisions themselves: a reason about the
//! step as a whole is one [`step_snap`] gives a placement whose window has
//! no reason of its own to snap, so it is the reason every placement of
//! the step gets.

use crate::WindowId;

use super::super::{ApplyFacts, ApplySnap, StepFacts, StepSnap, apply_snap, step_snap};
use super::{SnapLine, StepSnapLine};

/// `facts` for a window that has no reason of its own to snap: one that
/// does not appear, wait behind the monocle window or change monitors.
fn of_the_step(facts: &StepFacts) -> StepFacts {
    StepFacts {
        appears: false,
        behind_monocle: false,
        same_monitor: true,
        ..*facts
    }
}

impl StepSnapLine {
    /// The line of a step of `placements` placements, with `facts` about
    /// it, when all of them snap for a reason about the step as a whole.
    /// `None` when there is none, when the step has no placements, and
    /// when transitions are off, which the log does not repeat for every
    /// step.
    pub fn of_step(facts: &StepFacts, placements: usize) -> Option<Self> {
        match step_snap(&of_the_step(facts))? {
            StepSnap::Off => None,
            why => (placements > 0).then_some(Self { placements, why }),
        }
    }
}

impl SnapLine<StepSnap> {
    /// The line of a placement with `facts` that the step decision keeps
    /// from animating, of a window the step `moves`. `None` when it may
    /// animate, when the reason is about the step as a whole, which gets a
    /// line of its own, and when the window stays where it is.
    pub fn stepped(window: WindowId, facts: &StepFacts, moves: bool) -> Option<Self> {
        if step_snap(&of_the_step(facts)).is_some() {
            return None;
        }
        let why = step_snap(facts)?;
        moves.then_some(Self { window, why })
    }
}

impl SnapLine<ApplySnap> {
    /// The line of a placement with `facts` that snaps as it is applied.
    /// `None` when it still animates, and when it leaves its window where
    /// it is, which has nothing to snap.
    pub fn applied(window: WindowId, facts: &ApplyFacts) -> Option<Self> {
        let why = apply_snap(facts)?;
        (facts.from != facts.to).then_some(Self { window, why })
    }
}

#[cfg(test)]
#[path = "tests/journal_snap_tests.rs"]
mod tests;
