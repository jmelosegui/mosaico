//! Holds back the effects a platform may apply late.
//!
//! While the user is still pressing keys, only the latest focus change
//! and the latest coalesced position of each window matter. Applying
//! every intermediate one is slow (a focus change is a cross-process
//! call) and makes windows repaint once per key press.

use std::time::Instant;

use crate::{Rect, WindowId};

use super::{Effect, Timing};

/// Effects held back until input goes quiet. [`release_at`](super::release_at)
/// says when that is.
#[derive(Debug, Default)]
pub struct Coalescer {
    /// Coalesced placements, latest per window, in arrival order.
    places: Vec<(WindowId, Rect)>,
    /// Effects that must wait for a held placement of their window:
    /// its border, and the cursor moving onto it.
    after: Vec<Effect>,
    /// The latest focus change.
    focus: Option<Effect>,
    /// When the effects held now started being held. Stale while
    /// nothing is held.
    since: Option<Instant>,
}

impl Coalescer {
    /// Takes the next effect of a step made at `now`. Returns it when it
    /// should be applied right away, or `None` when it is being held.
    pub fn admit(&mut self, effect: Effect, now: Instant) -> Option<Effect> {
        match effect {
            Effect::Place {
                window,
                rect,
                timing: Timing::Coalesced,
            } => {
                self.hold(now);
                match self.places.iter_mut().find(|(w, _)| *w == window) {
                    Some(held) => held.1 = rect,
                    None => self.places.push((window, rect)),
                }
                None
            }
            Effect::Place { window, .. } => {
                // A placement made now supersedes a held one, along with
                // whatever was waiting for it.
                self.forget(window);
                Some(effect)
            }
            // The window is leaving the screen. What was held for it
            // would move a window that is gone, or draw a border around
            // it that nothing removes afterwards (#71).
            Effect::Hide { window, .. }
            | Effect::Minimize { window }
            | Effect::Close { window }
            | Effect::DropBorder { window } => {
                self.forget(window);
                Some(effect)
            }
            Effect::Focus { .. } => {
                self.hold(now);
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

    /// Drops the held placement of `window` and everything waiting for
    /// it. The platform calls this when the window is destroyed.
    pub fn forget(&mut self, window: WindowId) {
        self.places.retain(|&(w, _)| w != window);
        self.after.retain(|e| waits_for(e) != Some(window));
    }

    /// Whether anything is held.
    pub fn is_empty(&self) -> bool {
        self.focus.is_none() && self.places.is_empty()
    }

    /// When the effects held now started being held, or `None` when
    /// nothing is held.
    pub fn held_since(&self) -> Option<Instant> {
        if self.is_empty() { None } else { self.since }
    }

    /// Starts the hold at `now`, unless something is held already. Call
    /// it before holding the effect.
    fn hold(&mut self, now: Instant) {
        if self.is_empty() {
            self.since = Some(now);
        }
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

#[cfg(test)]
#[path = "tests/coalesce_forget_tests.rs"]
mod forget_tests;

#[cfg(test)]
#[path = "tests/coalesce_hold_tests.rs"]
mod hold_tests;
