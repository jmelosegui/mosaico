//! Holds back the effects the tiling manager may apply late.
//!
//! While the user is still pressing keys, only the latest focus change
//! and the latest coalesced position of each window matter. Applying
//! every intermediate one is slow (a focus change is a cross-process
//! call) and makes windows repaint once per key press.

use std::time::Instant;

use crate::{Rect, WindowId};

use super::{Effect, Timing, Transition};

#[path = "coalesce_hold.rs"]
mod hold;
#[path = "coalesce_release.rs"]
mod release;

/// Effects held back until input goes quiet. [`release_at`](super::release_at)
/// says when that is.
#[derive(Debug, Default)]
pub struct Coalescer {
    /// Coalesced placements, latest per window, in arrival order, each
    /// with whether it may animate.
    places: Vec<(WindowId, Rect, Transition)>,
    /// Effects that must wait for a held placement of their window:
    /// its border, the latest one only, and the cursor moving onto it.
    after: Vec<Effect>,
    /// The latest focus change.
    focus: Option<Effect>,
    /// Restacks of the window taking focus, which must land after it.
    behind: Vec<Effect>,
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
                transition,
            } => {
                self.hold(now);
                let place = (window, rect, transition);
                match self.places.iter_mut().find(|(w, ..)| *w == window) {
                    Some(held) => *held = place,
                    None => self.places.push(place),
                }
                None
            }
            Effect::Place { window, .. } => {
                // A placement made now supersedes a held one, along with
                // whatever was waiting for it. A focus change held for the
                // window still lands.
                self.forget_placement(window);
                Some(effect)
            }
            // The window is leaving the screen or the layout. What was
            // held for it would move or focus a window that is gone or no
            // longer tiled, or draw a border around it that nothing
            // removes afterwards (#71, #135).
            Effect::Hide { window, .. }
            | Effect::Minimize { window }
            | Effect::Close { window }
            | Effect::DropBorder { window }
            | Effect::Untile { window } => {
                self.forget(window);
                Some(effect)
            }
            Effect::Focus { .. } => {
                self.hold(now);
                self.focus = Some(effect);
                None
            }
            Effect::PutBehind { window, .. } if self.focuses(window) => {
                self.behind.push(effect);
                None
            }
            _ => match waits_for(&effect) {
                Some(window) if self.places.iter().any(|&(w, ..)| w == window) => {
                    self.hold_until_placed(effect);
                    None
                }
                _ => Some(effect),
            },
        }
    }

    /// Whether anything is held.
    pub fn is_empty(&self) -> bool {
        self.focus.is_none() && self.places.is_empty() && self.behind.is_empty()
    }

    /// Whether a focus change is held.
    pub fn holds_focus(&self) -> bool {
        self.focus.is_some()
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
}

/// The window whose held placement this effect has to wait for.
pub(super) fn waits_for(effect: &Effect) -> Option<WindowId> {
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

#[cfg(test)]
#[path = "tests/coalesce_border_tests.rs"]
mod border_tests;
