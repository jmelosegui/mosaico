//! Placements that animate: the motions they become, and their ends.
//!
//! The tiler marks the placements that may animate as it finishes a step
//! ([`Transition::Animate`]). As the applier applies one, it checks again
//! against what the desktop says now, with [`still_animates`]: the window
//! is then given a motion to its target instead of being placed there,
//! and the frame step moves it on. Any other placement of a window that is
//! moving ends its motion and places it at once, in full.

use std::time::Instant;

use crate::config::AnimationConfig;
use crate::motion::{ApplyFacts, Motions, still_animates};
use crate::{Rect, WindowId};

use super::super::window_effects::place_and_repaint;
use super::super::{Desktop, Effect, Platform, Transition, apply_window_effect};
use super::Applier;

/// What the applier keeps for the placements that animate.
#[derive(Debug)]
pub(super) struct Transitions {
    /// The windows moving to their placements.
    pub(super) motions: Motions,
    /// How placements animate, as configured.
    pub(super) animations: AnimationConfig,
    /// Whether the platform can move windows through frames.
    pub(super) frames_supported: bool,
}

impl Default for Transitions {
    fn default() -> Self {
        Self {
            motions: Motions::default(),
            animations: AnimationConfig::default(),
            frames_supported: true,
        }
    }
}

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

    /// When a frame is due even if no display frame comes, after `after`,
    /// the last frame step: when the first motion is due to end. `None`
    /// while nothing is due.
    pub(in crate::tiler) fn frame_due(&self, after: Option<Instant>) -> Option<Instant> {
        let due = self.transitions.motions.deadline();
        due.filter(|&due| after.is_none_or(|after| due > after))
    }

    /// Turns transitions off for the rest of the session, as on a platform
    /// that cannot move windows frame by frame: nothing animates from then
    /// on, and every moving window is placed at its target at once.
    pub(in crate::tiler) fn turn_off_transitions(&mut self, platform: &mut dyn Platform) {
        self.transitions.frames_supported = false;
        self.frame(platform);
    }

    /// The frame step: ends every motion at its target, placing each
    /// window there in full.
    pub(in crate::tiler) fn frame(&mut self, platform: &mut dyn Platform) {
        self.finish_motions(platform);
    }

    /// Applies a placement that is not made in full: as a motion when it
    /// may still animate, otherwise as it always was, unless its window is
    /// moving: the motion ends, and the window is placed in full.
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
            && let Some(from) = self.animates_from(window, rect, platform.desktop(), now)
        {
            let spec = self.transitions.animations.spec();
            self.transitions
                .motions
                .retarget(window, from, rect, now, spec);
        } else if self.transitions.motions.end(window).is_some() {
            place_and_repaint(window, rect, platform.control());
        } else {
            apply_window_effect(effect, platform.desktop(), platform.control());
        }
    }

    /// Places a window in full, ending its motion if it has one.
    pub(super) fn place_now(&mut self, window: WindowId, rect: Rect, platform: &mut dyn Platform) {
        self.stop_motion(window);
        place_and_repaint(window, rect, platform.control());
    }

    /// Where a placement of `window` at `to` would animate from, if it may
    /// still animate as it is applied at `now`, and no display change
    /// settles: from where its motion has it, or where the desktop has it.
    fn animates_from(
        &self,
        window: WindowId,
        to: Rect,
        desktop: &dyn Desktop,
        now: Instant,
    ) -> Option<Rect> {
        let transitions = &self.transitions;
        if !transitions.animations.is_on() || self.display.settling {
            return None;
        }
        let moving = transitions.motions.rect_at(window, now);
        let from = moving.or_else(|| desktop.rect(window))?;
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
        still_animates(&facts).then_some(from)
    }
}

#[cfg(test)]
#[path = "tests/transitions.rs"]
mod tests;
