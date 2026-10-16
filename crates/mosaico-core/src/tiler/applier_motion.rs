//! Placements that animate: the motions they become.
//!
//! The tiler marks the placements that may animate as it finishes a step
//! ([`Transition::Animate`]). As the applier applies one, it checks again
//! against what the desktop says now, with [`still_animates`]: the window
//! is then given a motion to its target instead of being placed there,
//! and the frame step moves it on (`applier_frames.rs`). Any other
//! placement of a window that is moving ends its motion and places it at
//! once, in full, behind the frames posted to it before.

use std::time::Instant;

use crate::config::AnimationConfig;
use crate::motion::{ApplyFacts, ApplySnap, Ending, SnapLine, still_animates};
use crate::{Rect, WindowId};

use super::super::{Effect, Platform, Transition};
use super::Applier;

#[path = "applier_transitions.rs"]
mod transitions;

pub(super) use transitions::Transitions;

impl Applier {
    /// Takes the `[animations]` settings, at start and on every reload.
    /// A window already moving keeps the duration and easing it started
    /// with while its target stays. One sent elsewhere turns with the new
    /// ones, and with transitions off it is placed there at once.
    pub(in crate::tiler) fn set_animations(&mut self, animations: &AnimationConfig) {
        self.transitions.animations = animations.clone();
    }

    /// Whether any window is moving.
    pub(in crate::tiler) fn is_moving(&self) -> bool {
        !self.transitions.motions.is_empty()
    }

    /// Applies a placement that is not made in full: as a motion when it
    /// may still animate, which ends the wait for the window to land where
    /// it was placed before, otherwise as it always was, unless its window
    /// is moving: the motion ends, and the window is placed in full. A
    /// window something may still be on its way to is placed as
    /// `applier_posted.rs` says.
    pub(super) fn run_place(&mut self, effect: &Effect, platform: &mut dyn Platform, now: Instant) {
        let Effect::Place {
            window,
            rect,
            transition,
            ..
        } = *effect
        else {
            return;
        };
        if transition == Transition::Animate
            && let Some(from) = self.animates_from(window, rect, platform, now)
        {
            let spec = self.transitions.animations.spec();
            self.forget_landing(window);
            self.border_moves(window, from, rect);
            // A target the window already has changes nothing.
            let turns = self.transitions.motions.target(window) != Some(rect);
            let transitions = &mut self.transitions;
            transitions.motions.retarget(window, from, rect, now, spec);
            if turns {
                transitions.journal.moved(window, now, spec);
            }
        } else if self.transitions.motions.is_moving(window) {
            self.place_now(window, rect, platform, now);
        } else {
            self.place_unless_there(window, rect, platform);
        }
    }

    /// Places a window in full at `now`, ending its motion if it has one,
    /// which the log hears of with how the placement was made. The
    /// border it kept for the motion is left to the effects that follow.
    pub(super) fn place_now(
        &mut self,
        window: WindowId,
        rect: Rect,
        platform: &mut dyn Platform,
        now: Instant,
    ) {
        self.drop_motion(window);
        let placed = self.send_to(window, rect, platform);
        self.log_end(window, Ending::NewTarget, Some(placed), now);
    }

    /// Ends the window's motion where the window is, and forgets what was
    /// kept for it: its border, and the target a display change kept.
    pub(super) fn drop_motion(&mut self, window: WindowId) {
        self.transitions.motions.end(window);
        self.forget_kept_border(window);
        self.forget_stopped(window);
    }

    /// Where a placement of `window` at `to` would animate from, if it may
    /// still animate as it is applied at `now`: from where the window is as
    /// far as its motion knows, or where it was last sent while that may be
    /// on its way, or from where the desktop has it. When it may not, the
    /// log is told why, unless the window stays where it is; while a display
    /// change settles, before anything is asked of the window.
    fn animates_from(
        &mut self,
        window: WindowId,
        to: Rect,
        platform: &dyn Platform,
        now: Instant,
    ) -> Option<Rect> {
        if !self.transitions.animations.is_on() {
            return None;
        }
        if self.display.settling {
            let why = ApplySnap::DisplaysChanging;
            self.log(SnapLine { window, why });
            return None;
        }
        let desktop = platform.desktop();
        let transitions = &self.transitions;
        let moving = transitions.motions.position(window, now);
        let sent = || self.last_sent(window, platform.control());
        let from = moving.or_else(sent).or_else(|| desktop.rect(window))?;
        let follow_system = transitions.animations.follow_system;
        let facts = ApplyFacts {
            follow_system,
            // The system's setting counts only while the config follows it.
            system_allows: !follow_system || desktop.animations_allowed(),
            minimized: desktop.is_minimized(window),
            maximized: desktop.is_maximized(window),
            frames_supported: transitions.frames_supported,
            from,
            to,
        };
        if still_animates(&facts) {
            return Some(from);
        }
        if let Some(line) = SnapLine::applied(window, &facts) {
            self.log(line);
        }
        None
    }
}

#[cfg(test)]
#[path = "tests/transitions.rs"]
mod tests;
