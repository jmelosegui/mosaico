//! The step decision: which placements of a step may animate, decided
//! as the step finishes, from what the step did.

use std::collections::HashSet;

use crate::motion::{StepFacts, step_animates};
use crate::{MonitorId, Rect, WindowId};

use super::super::{Desktop, Effect, Timing, Transition};
use super::Plan;

/// What the step decision needs to know of a step as a whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tiler) struct StepKind {
    /// Whether transitions are on: enabled, with a duration above zero.
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
/// animate. Every other placement snaps.
pub(super) fn mark(
    effects: &mut [Effect],
    placements: &Placements,
    step: &StepKind,
    desktop: &dyn Desktop,
) {
    // Nothing animates then, whatever else holds, so the desktop is not
    // asked about any window.
    if !step.enabled || step.rebuild {
        return;
    }
    let appearing = appearing(effects, &placements.taken_in);
    let windows_moved = windows_moved(effects, desktop);
    for &(at, monitor) in &placements.monitors {
        let Effect::Place { window, .. } = effects[at] else {
            continue;
        };
        let facts = StepFacts {
            enabled: step.enabled,
            appears: appearing.contains(&window),
            behind_monocle: waits_behind(&effects[..at], window),
            rebuild: step.rebuild,
            same_monitor: desktop.monitor_of(window) == monitor,
            windows_moved,
        };
        if step_animates(&facts)
            && let Effect::Place { transition, .. } = &mut effects[at]
        {
            *transition = Transition::Animate;
        }
    }
}

/// The windows the step shows, moves ahead of the layout or takes in.
fn appearing(effects: &[Effect], taken_in: &HashSet<WindowId>) -> HashSet<WindowId> {
    let shown = effects.iter().filter_map(|effect| match *effect {
        Effect::Show { window, .. } | Effect::Reveal { window } => Some(window),
        Effect::PrePlace { window, .. } => Some(window),
        _ => None,
    });
    taken_in.iter().copied().chain(shown).collect()
}

/// Whether `window` is put behind another in `earlier`, the effects
/// before its placement: it waits behind the window in monocle.
fn waits_behind(earlier: &[Effect], window: WindowId) -> bool {
    let put_behind = |e: &Effect| matches!(*e, Effect::PutBehind { window: w, .. } if w == window);
    earlier.iter().any(put_behind)
}

/// How many windows the step's placements move from where the desktop
/// has them: a placement that leaves its window where it is moves none.
fn windows_moved(effects: &[Effect], desktop: &dyn Desktop) -> usize {
    let mut moved = HashSet::new();
    for effect in effects {
        if let Effect::Place { window, rect, .. } = *effect
            && desktop.rect(window) != Some(rect)
        {
            moved.insert(window);
        }
    }
    moved.len()
}

#[cfg(test)]
#[path = "tests/plan_transitions_tests.rs"]
mod tests;
