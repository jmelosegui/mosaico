//! The step decision: which placements of a step may animate, decided
//! as the step finishes, from what the step did.

use std::collections::HashSet;

use crate::motion::{SnapLine, StepFacts, StepSnapLine, step_animates};
use crate::{MonitorId, Rect, WindowId};

use super::super::{Desktop, Effect, Timing, Transition};
use super::Plan;

#[path = "plan_transitions_facts.rs"]
mod facts;
#[path = "plan_transitions_log.rs"]
mod said;

use facts::{appearing, moved, waits_behind};
pub(super) use said::{Said, mark_logged};

/// What the step decision needs to know of a step as a whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tiler) struct StepKind {
    /// Whether transitions are on: enabled, with a duration above zero, in
    /// a session that has frames.
    pub(in crate::tiler) enabled: bool,
    /// Whether the step lays the windows out from scratch: a display
    /// change, the tiler's start, or the bar reservation right after it.
    pub(in crate::tiler) rebuild: bool,
}

/// What a step did, beyond its effects, that decides which of its
/// placements may animate.
#[derive(Debug, Default)]
pub(super) struct Placements {
    /// For each placement of a layout pass, where it is in the step's
    /// effects, and the monitor it lays its window out on.
    monitors: Vec<(usize, MonitorId)>,
    /// The windows the step took in.
    taken_in: HashSet<WindowId>,
}

impl Plan<'_> {
    /// Places `window` at `rect`, laying it out on the monitor `monitor`.
    /// The placement snaps, unless the step decision marks it as one that
    /// may animate when the step finishes.
    pub(in crate::tiler) fn place(
        &mut self,
        window: WindowId,
        rect: Rect,
        timing: Timing,
        monitor: MonitorId,
    ) {
        let at = self.effects.len();
        self.placements.monitors.push((at, monitor));
        self.emit(Effect::Place {
            window,
            rect,
            timing,
            transition: Transition::Snap,
        });
    }

    /// Notes that the step takes `window` in, so it appears where the
    /// layout puts it instead of moving there.
    pub(in crate::tiler) fn take_in(&mut self, window: WindowId) {
        self.placements.taken_in.insert(window);
    }
}

/// Marks each placement of a layout pass that [`step_animates`] lets
/// animate. Every other placement snaps. Returns what the log is told of
/// it, from the decision itself: why the step as a whole snaps, or why
/// each placement that moves its window does.
pub(super) fn mark(
    effects: &mut [Effect],
    placements: &Placements,
    step: &StepKind,
    desktop: &dyn Desktop,
) -> Vec<Said> {
    let count = placements.monitors.len();
    // Nothing animates then, whatever else holds, so the desktop is not
    // asked about any window.
    if !step.enabled || step.rebuild {
        let line = StepSnapLine::of_step(&of_step(step, 0), count);
        return line.map(Said::Step).into_iter().collect();
    }
    let appearing = appearing(effects, &placements.taken_in);
    let moved = moved(effects, desktop);
    let whole = of_step(step, moved.len());
    let mut said: Vec<Said> = StepSnapLine::of_step(&whole, count)
        .map(Said::Step)
        .into_iter()
        .collect();
    for &(at, monitor) in &placements.monitors {
        let Effect::Place { window, .. } = effects[at] else {
            continue;
        };
        let facts = StepFacts {
            appears: appearing.contains(&window),
            behind_monocle: waits_behind(&effects[..at], window),
            same_monitor: desktop.monitor_of(window) == monitor,
            ..whole
        };
        if step_animates(&facts) {
            if let Effect::Place { transition, .. } = &mut effects[at] {
                *transition = Transition::Animate;
            }
        } else {
            let line = SnapLine::stepped(window, &facts, moved.contains(&window));
            said.extend(line.map(Said::Placement));
        }
    }
    said
}

/// The facts of `step`, which moves `windows_moved` windows, for a
/// placement of a window with no reason of its own to snap.
fn of_step(step: &StepKind, windows_moved: usize) -> StepFacts {
    StepFacts {
        enabled: step.enabled,
        appears: false,
        behind_monocle: false,
        rebuild: step.rebuild,
        same_monitor: true,
        windows_moved,
    }
}

#[cfg(test)]
#[path = "tests/plan_transitions_tests.rs"]
mod tests;
