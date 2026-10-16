//! What the step decision reads from the effects of a step.

use std::collections::HashSet;

use crate::WindowId;

use super::super::super::{Desktop, Effect};

/// The windows the step shows, moves ahead of the layout or takes in.
pub(super) fn appearing(effects: &[Effect], taken_in: &HashSet<WindowId>) -> HashSet<WindowId> {
    let shown = effects.iter().filter_map(|effect| match *effect {
        Effect::Show { window, .. } | Effect::Reveal { window } => Some(window),
        Effect::PrePlace { window, .. } => Some(window),
        _ => None,
    });
    taken_in.iter().copied().chain(shown).collect()
}

/// Whether `window` is put behind another in `earlier`, the effects
/// before its placement: it waits behind the window in monocle.
pub(super) fn waits_behind(earlier: &[Effect], window: WindowId) -> bool {
    let put_behind = |e: &Effect| matches!(*e, Effect::PutBehind { window: w, .. } if w == window);
    earlier.iter().any(put_behind)
}

/// The windows the step's placements move from where the desktop has
/// them: a placement that leaves its window where it is moves none.
pub(super) fn moved(effects: &[Effect], desktop: &dyn Desktop) -> HashSet<WindowId> {
    let mut moved = HashSet::new();
    for effect in effects {
        if let Effect::Place { window, rect, .. } = *effect
            && desktop.rect(window) != Some(rect)
        {
            moved.insert(window);
        }
    }
    moved
}
