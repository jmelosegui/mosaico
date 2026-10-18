//! What a click on a widget of the bar runs: the `[[mouse.click]]`
//! bindings, over the defaults.

use crate::Action;
use crate::config::{BarMouseConfig, ClickBinding, WidgetKind};

use super::{BarSide, Button};

#[path = "clicks_rejected.rs"]
mod rejected;

pub use rejected::{ClickProblem, RejectedClick};

/// The action each button runs on each kind of widget.
///
/// Built from the `[mouse]` section of `bar.toml`: the defaults
/// ([`ClickBinding::defaults`]), then each `[[mouse.click]]` entry in the
/// order the file lists them, which replaces the binding of the same
/// widget and button, adds one, or with [`ClickBinding::NONE`] removes
/// it. An entry the bar cannot run is left out as if it were not there,
/// so the binding of its widget and button stays as it was (see
/// [`rejected`](Self::rejected)).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WidgetClicks {
    bindings: Vec<(WidgetKind, Button, Action)>,
}

impl WidgetClicks {
    /// What clicks on the widgets run with `mouse`: nothing while
    /// `[mouse] enabled` is off.
    pub fn new(mouse: &BarMouseConfig) -> Self {
        let (mut clicks, _) = resolve(mouse);
        if !mouse.enabled {
            clicks.bindings.clear();
        }
        clicks
    }

    /// The `[[mouse.click]]` entries of `mouse` the bar leaves out, in the
    /// order the file lists them, and why: a missing or unknown widget,
    /// button or action, or an action about the focused window. The bar
    /// may only run an action that acts on a monitor, its own, or on the
    /// whole daemon. An entry that removes a binding is not left out.
    pub fn rejected(mouse: &BarMouseConfig) -> Vec<RejectedClick> {
        resolve(mouse).1
    }

    /// How many bindings there are: buttons that run an action on a kind
    /// of widget.
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Whether no click on a widget runs anything.
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// The action a click of `button` runs on a widget of `kind` drawn on
    /// `side`, if it runs one.
    ///
    /// The workspaces widget takes clicks on the left side alone, where it
    /// shows the workspace labels. On the right it is a pill showing its
    /// icon, which takes none.
    pub fn action(&self, kind: WidgetKind, side: BarSide, button: Button) -> Option<Action> {
        if kind == WidgetKind::Workspaces && side == BarSide::Right {
            return None;
        }
        self.bindings
            .iter()
            .find(|&&(k, b, _)| (k, b) == (kind, button))
            .map(|&(_, _, action)| action)
    }

    /// Whether a click of some button runs an action on a widget of `kind`
    /// drawn on `side`.
    pub fn clickable(&self, kind: WidgetKind, side: BarSide) -> bool {
        [Button::Left, Button::Right, Button::Middle]
            .into_iter()
            .any(|button| self.action(kind, side, button).is_some())
    }

    /// Has a click of `button` on `kind` run `action`, in place of what it
    /// ran, or nothing for `None`.
    fn set(&mut self, kind: WidgetKind, button: Button, action: Option<Action>) {
        let bound = self
            .bindings
            .iter()
            .position(|&(k, b, _)| (k, b) == (kind, button));
        match (bound, action) {
            (Some(at), Some(action)) => self.bindings[at].2 = action,
            (Some(at), None) => {
                self.bindings.remove(at);
            }
            (None, Some(action)) => self.bindings.push((kind, button, action)),
            (None, None) => {}
        }
    }
}

/// The bindings of `mouse` over the defaults, and the entries left out.
fn resolve(mouse: &BarMouseConfig) -> (WidgetClicks, Vec<RejectedClick>) {
    let mut clicks = WidgetClicks::default();
    let mut left_out = Vec::new();
    let defaults = ClickBinding::defaults();
    for binding in defaults.iter().chain(&mouse.click) {
        match rejected::check(binding) {
            Ok((kind, button, action)) => clicks.set(kind, button, action),
            Err(problem) => left_out.push(RejectedClick {
                binding: binding.clone(),
                problem,
            }),
        }
    }
    (clicks, left_out)
}

#[cfg(test)]
#[path = "tests/clicks_support.rs"]
mod support;

#[cfg(test)]
#[path = "tests/clicks_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/clicks_merge_tests.rs"]
mod merge_tests;

#[cfg(test)]
#[path = "tests/clicks_unbind_tests.rs"]
mod unbind_tests;

#[cfg(test)]
#[path = "tests/clicks_rejected_tests.rs"]
mod rejected_tests;

#[cfg(test)]
#[path = "tests/clicks_keys_tests.rs"]
mod keys_tests;
