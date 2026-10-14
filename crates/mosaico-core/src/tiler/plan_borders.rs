//! At most one border per window in a step.
//!
//! A step often decides a window's border more than once: when the
//! layout places the window, again when every border is brought in line
//! with the layout, and again after focus moves. Only the last decision
//! is what the screen should show, and drawing a border is not cheap, so
//! a step keeps that one, where it was.

use std::collections::HashMap;

use super::Effect;

/// `effects` with only the last [`Effect::Border`] of each window, left
/// where it was. A border that a [`Effect::DropBorder`] of its window
/// follows goes too, since the overlay is destroyed after it.
///
/// Keeping the last one, rather than the first, keeps every border after
/// the placement it was decided for, so a border never lands at a slot
/// its window has not reached yet. It also lets hiding or dropping a
/// border win over an earlier draw in the same step.
pub(super) fn keep_last_borders(effects: Vec<Effect>) -> Vec<Effect> {
    let mut last = HashMap::new();
    for (i, effect) in effects.iter().enumerate() {
        if let Effect::Border { window, .. } | Effect::DropBorder { window } = *effect {
            last.insert(window, i);
        }
    }
    effects
        .into_iter()
        .enumerate()
        .filter(|(i, effect)| match *effect {
            Effect::Border { window, .. } => last.get(&window) == Some(i),
            _ => true,
        })
        .map(|(_, effect)| effect)
        .collect()
}

#[cfg(test)]
#[path = "tests/plan_borders_tests.rs"]
mod tests;
