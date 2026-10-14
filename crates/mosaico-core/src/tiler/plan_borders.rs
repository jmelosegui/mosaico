//! At most one border per window in a step.
//!
//! A step often decides a window's border more than once: when the
//! layout places the window, again when every border is brought in line
//! with the layout, and again after focus moves. Only the last decision
//! is what the screen should show, and drawing a border is not cheap, so
//! a step draws that one, once, where the window's border lands with it.

use std::collections::HashMap;

use crate::WindowId;

use super::Effect;

/// `effects` with one [`Effect::Border`] per window: the style of its
/// last one, where its first one after the window's last placement was.
///
/// - The style is the last one decided, so a hide wins over an earlier
///   draw, and a draw over an earlier hide.
/// - It goes right where the step first drew the border after placing
///   the window, so each border still lands with its own window, not
///   after every window the step moves. A border is never drawn before
///   the placement it was decided for.
/// - A window whose border a [`Effect::DropBorder`] follows has none:
///   the overlay is destroyed after it.
pub(super) fn keep_last_borders(effects: Vec<Effect>) -> Vec<Effect> {
    let targets = targets(&effects);
    let mut slots: Vec<Option<Effect>> = effects.into_iter().map(Some).collect();
    for &(at, from) in targets.values() {
        if at != from {
            slots[at] = slots[from].take();
        }
    }
    slots
        .into_iter()
        .enumerate()
        .filter_map(|(i, slot)| {
            let effect = slot?;
            match effect {
                Effect::Border { window, .. } => {
                    (targets.get(&window).map(|&(at, _)| at) == Some(i)).then_some(effect)
                }
                _ => Some(effect),
            }
        })
        .collect()
}

/// For each window that keeps a border: where its border goes and which
/// of its borders gives the style, as positions in `effects`.
fn targets(effects: &[Effect]) -> HashMap<WindowId, (usize, usize)> {
    let mut last = HashMap::new();
    for (i, effect) in effects.iter().enumerate() {
        if let Effect::Border { window, .. } = *effect {
            last.insert(window, i);
        }
    }
    let mut targets = HashMap::new();
    for (window, from) in last {
        let dropped = effects[from..]
            .iter()
            .any(|e| matches!(*e, Effect::DropBorder { window: w } if w == window));
        if dropped {
            continue;
        }
        let start = effects[..from]
            .iter()
            .rposition(|e| {
                matches!(*e, Effect::Place { window: w, .. } | Effect::DropBorder { window: w }
                    if w == window)
            })
            .map_or(0, |anchor| anchor + 1);
        let at = effects[start..]
            .iter()
            .position(|e| matches!(*e, Effect::Border { window: w, .. } if w == window))
            .map_or(from, |offset| start + offset);
        targets.insert(window, (at, from));
    }
    targets
}

#[cfg(test)]
#[path = "tests/plan_borders_tests.rs"]
mod tests;
