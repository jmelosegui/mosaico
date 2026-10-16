//! Holds back the effects the tiling manager may apply late.
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
    /// its border, the latest one only, and the cursor moving onto it.
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
            _ => match waits_for(&effect) {
                Some(window) if self.places.iter().any(|&(w, _)| w == window) => {
                    self.hold_until_placed(effect);
                    None
                }
                _ => Some(effect),
            },
        }
    }

    /// Holds an effect until its window's placement is applied.
    ///
    /// A border takes the place of the one already held for its window,
    /// so only the latest style is drawn. Its rect may still be the slot
    /// the window is leaving, when a step that did not move the window
    /// decided it, so [`take`](Self::take) sets it from the placement.
    fn hold_until_placed(&mut self, effect: Effect) {
        if let Effect::Border { window, .. } = effect
            && let Some(held) = self
                .after
                .iter_mut()
                .find(|e| matches!(e, Effect::Border { window: w, .. } if *w == window))
        {
            *held = effect;
            return;
        }
        self.after.push(effect);
    }

    /// Drops everything held for `window`: its placement, what waits for
    /// it, and a focus change to it. The window is gone, or leaving the
    /// screen or the layout; the tiling manager calls this when it is
    /// destroyed.
    pub fn forget(&mut self, window: WindowId) {
        self.forget_placement(window);
        if matches!(self.focus, Some(Effect::Focus { window: w, .. }) if w == window) {
            self.focus = None;
        }
    }

    /// Drops the held placement of `window` and everything waiting for
    /// it.
    fn forget_placement(&mut self, window: WindowId) {
        self.places.retain(|&(w, _)| w != window);
        self.after.retain(|e| waits_for(e) != Some(window));
    }

    /// Whether anything is held.
    pub fn is_empty(&self) -> bool {
        self.focus.is_none() && self.places.is_empty()
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

    /// Releases everything held, in the order to apply it: placements,
    /// then what waited for them, then the focus change.
    ///
    /// A border is drawn around its window's held placement. A step that
    /// did not move the window works the rect out from the desktop, where
    /// the move has not happened yet (#133).
    pub fn take(&mut self) -> Vec<Effect> {
        let places = std::mem::take(&mut self.places);
        let mut out: Vec<Effect> = places
            .iter()
            .map(|&(window, rect)| Effect::Place {
                window,
                rect,
                timing: Timing::Now,
            })
            .collect();
        for mut effect in self.after.drain(..) {
            if let Effect::Border {
                window,
                style: Some(ref mut style),
            } = effect
                && let Some(&(_, rect)) = places.iter().find(|&&(w, _)| w == window)
            {
                style.rect = rect;
            }
            out.push(effect);
        }
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

#[cfg(test)]
#[path = "tests/coalesce_border_tests.rs"]
mod border_tests;
