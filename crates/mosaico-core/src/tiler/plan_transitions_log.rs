//! What the step decision tells the log.

use std::fmt;

use crate::motion::{SnapLine, StepSnap, StepSnapLine};

use super::super::super::{Desktop, Effect};
use super::{Placements, StepKind, mark};

/// A line the step decision gives the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tiler) enum Said {
    /// Every placement of the step snaps, for a reason about the step.
    Step(StepSnapLine),
    /// A placement that moves its window snaps, for a reason of its own.
    Placement(SnapLine<StepSnap>),
}

impl fmt::Display for Said {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Step(line) => line.fmt(f),
            Self::Placement(line) => line.fmt(f),
        }
    }
}

/// Marks the placements of a layout pass as [`mark`] does, and logs what
/// it says, for debugging.
pub(in crate::tiler) fn mark_logged(
    effects: &mut [Effect],
    placements: &Placements,
    step: &StepKind,
    desktop: &dyn Desktop,
) {
    for line in mark(effects, placements, step, desktop) {
        crate::log_debug!("{line}");
    }
}
