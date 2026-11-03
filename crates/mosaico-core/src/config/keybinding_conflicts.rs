//! Keybindings that claim the same key combination.

use std::collections::HashMap;

use crate::hotkey::KeyCombo;

use super::{Keybinding, LaunchBinding};

/// A key combination that more than one binding is bound to, so only one
/// of them can be registered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    /// The combination they share.
    pub combo: KeyCombo,
    /// The positions of the keybindings that share it, in the order given.
    pub bindings: Vec<usize>,
    /// The positions of the launch bindings that share it, in the order
    /// given.
    pub launches: Vec<usize>,
}

/// The combinations that two or more of `bindings` share, in the order
/// their first binding appears.
///
/// Combinations are compared as the daemon registers them: the order and
/// case of the modifiers and of the key name do not matter, so `Alt+Shift+J`
/// written as `["shift", "alt"]` and `"j"` clashes with `["alt", "shift"]`
/// and `"J"`. A binding whose key name is not a key has no combination
/// and clashes with nothing.
pub fn conflicts(bindings: &[Keybinding]) -> Vec<Conflict> {
    conflicts_with_launches(bindings, &[])
}

/// The combinations that two or more bindings share, keybindings and
/// launch bindings alike, compared as [`conflicts`] compares them. They
/// come in the order their first binding appears, keybindings before
/// launch bindings.
pub fn conflicts_with_launches(
    bindings: &[Keybinding],
    launches: &[LaunchBinding],
) -> Vec<Conflict> {
    let combos = bindings
        .iter()
        .map(|b| (false, b.combo()))
        .chain(launches.iter().map(|l| (true, l.combo())));
    let mut order: Vec<Conflict> = Vec::new();
    let mut seen: HashMap<KeyCombo, usize> = HashMap::new();
    let (mut binding, mut launch) = (0, 0);
    for (is_launch, combo) in combos {
        let index = if is_launch { &mut launch } else { &mut binding };
        let position = *index;
        *index += 1;
        let Ok(combo) = combo else {
            continue;
        };
        let at = *seen.entry(combo).or_insert_with(|| {
            order.push(Conflict {
                combo,
                bindings: Vec::new(),
                launches: Vec::new(),
            });
            order.len() - 1
        });
        let positions = if is_launch {
            &mut order[at].launches
        } else {
            &mut order[at].bindings
        };
        positions.push(position);
    }
    order.retain(|c| c.bindings.len() + c.launches.len() > 1);
    order
}

#[cfg(test)]
#[path = "keybinding_conflicts_tests.rs"]
mod tests;
