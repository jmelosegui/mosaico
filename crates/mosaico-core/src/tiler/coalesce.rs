//! Holds back the effects a platform may apply late.
//!
//! While the user is still pressing keys, only the latest focus change
//! and the latest coalesced position of each window matter. Applying
//! every intermediate one is slow (a focus change is a cross-process
//! call) and makes windows repaint once per key press.

use crate::{Rect, WindowId};

use super::{Effect, Timing};

/// Effects held back until input goes quiet.
#[derive(Debug, Default)]
pub struct Coalescer {
    /// Coalesced placements, latest per window, in arrival order.
    places: Vec<(WindowId, Rect)>,
    /// Effects that must wait for a held placement of their window:
    /// its border, and the cursor moving onto it.
    after: Vec<Effect>,
    /// The latest focus change.
    focus: Option<Effect>,
}

impl Coalescer {
    /// Takes the next effect of a step. Returns it when it should be
    /// applied right away, or `None` when it is being held.
    pub fn admit(&mut self, effect: Effect) -> Option<Effect> {
        match effect {
            Effect::Place {
                window,
                rect,
                timing: Timing::Coalesced,
            } => {
                match self.places.iter_mut().find(|(w, _)| *w == window) {
                    Some(held) => held.1 = rect,
                    None => self.places.push((window, rect)),
                }
                None
            }
            Effect::Place { window, .. } => {
                // A placement made now supersedes a held one, along with
                // whatever was waiting for it.
                self.places.retain(|&(w, _)| w != window);
                self.after.retain(|e| waits_for(e) != Some(window));
                Some(effect)
            }
            Effect::Focus { .. } => {
                self.focus = Some(effect);
                None
            }
            _ => match waits_for(&effect) {
                Some(window) if self.places.iter().any(|&(w, _)| w == window) => {
                    self.after.push(effect);
                    None
                }
                _ => Some(effect),
            },
        }
    }

    /// Whether anything is held.
    pub fn is_empty(&self) -> bool {
        self.focus.is_none() && self.places.is_empty()
    }

    /// Releases everything held, in the order to apply it: placements,
    /// then what waited for them, then the focus change.
    pub fn take(&mut self) -> Vec<Effect> {
        let mut out: Vec<Effect> = std::mem::take(&mut self.places)
            .into_iter()
            .map(|(window, rect)| Effect::Place {
                window,
                rect,
                timing: Timing::Now,
            })
            .collect();
        out.append(&mut self.after);
        out.extend(self.focus.take());
        out
    }
}

/// The window whose held placement this effect has to wait for.
fn waits_for(effect: &Effect) -> Option<WindowId> {
    match *effect {
        Effect::Border { window, .. } | Effect::CenterCursor { window } => Some(window),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/coalesce_tests.rs"]
mod tests;
