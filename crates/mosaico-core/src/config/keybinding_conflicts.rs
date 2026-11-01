//! Keybindings that claim the same key combination.

use std::collections::HashMap;

use crate::hotkey::KeyCombo;

use super::Keybinding;

/// A key combination that more than one keybinding is bound to, so only
/// one of them can be registered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    /// The combination they share.
    pub combo: KeyCombo,
    /// The positions of the bindings that share it, in the order given.
    pub bindings: Vec<usize>,
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
    let mut order: Vec<Conflict> = Vec::new();
    let mut seen: HashMap<KeyCombo, usize> = HashMap::new();
    for (index, binding) in bindings.iter().enumerate() {
        let Ok(combo) = binding.combo() else {
            continue;
        };
        match seen.get(&combo) {
            Some(&at) => order[at].bindings.push(index),
            None => {
                seen.insert(combo, order.len());
                order.push(Conflict {
                    combo,
                    bindings: vec![index],
                });
            }
        }
    }
    order.retain(|c| c.bindings.len() > 1);
    order
}

#[cfg(test)]
#[path = "keybinding_conflicts_tests.rs"]
mod tests;
