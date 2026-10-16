//! Placements that animate: the motions they become.
//!
//! The tiler marks the placements that may animate as it finishes a step
//! ([`Transition::Animate`]). As the applier applies one, it checks again
//! against what the desktop says now, with [`still_animates`]: the window
//! is then given a motion to its target instead of being placed there,
//! and the frame step moves it on (`applier_frames.rs`). Any other
//! placement of a window that is moving ends its motion and places it at
//! once, in full, behind the frames posted to it before.

use std::collections::HashMap;
use std::time::Instant;

use crate::config::AnimationConfig;
use crate::frames::FrameOutcome;
use crate::motion::{ApplyFacts, Motions, still_animates};
use crate::{Rect, WindowId};

use super::super::{Effect, Platform, Transition};
use super::Applier;
use super::borders::MotionBorder;

/// What the applier keeps for the placements that animate.
#[derive(Debug)]
pub(super) struct Transitions {
    /// The windows moving to their placements.
    pub(super) motions: Motions,
    /// What the border of each moving window does until it is in place.
    pub(super) borders: HashMap<WindowId, MotionBorder>,
    /// What the platform found of each window on the last frame, kept from
    /// one frame to the next.
    pub(super) outcomes: Vec<FrameOutcome>,
    /// How placements animate, as configured.
    pub(super) animations: AnimationConfig,
    /// Whether the platform can move windows through frames.
    pub(super) frames_supported: bool,
    /// Whether the last frames failed to be posted, for a reason other than
    /// the platform not having frames.
    pub(super) frames_failing: bool,
}

impl Default for Transitions {
    fn default() -> Self {
        Self {
            motions: Motions::default(),
            borders: HashMap::new(),
            outcomes: Vec::new(),
            animations: AnimationConfig::default(),
            frames_supported: true,
            frames_failing: false,
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
            self.transitions
                .motions
                .retarget(window, from, rect, now, spec);
        } else if self.transitions.motions.is_moving(window) {
            self.place_now(window, rect, platform);
        } else {
            self.place_unless_there(window, rect, platform);
        }
    }

    /// Places a window in full, ending its motion if it has one. The
    /// border it kept for the motion is left to the effects that follow.
    pub(super) fn place_now(&mut self, window: WindowId, rect: Rect, platform: &mut dyn Platform) {
        self.stop_motion(window);
        self.send_to(window, rect, platform);
    }

    /// Where a placement of `window` at `to` would animate from, if it may
    /// still animate as it is applied at `now`, and no display change
    /// settles: from where the window is as far as its motion knows, or
    /// where it was last sent while that may be on its way, or from where
    /// the desktop has it.
    fn animates_from(
        &self,
        window: WindowId,
        to: Rect,
        platform: &dyn Platform,
        now: Instant,
    ) -> Option<Rect> {
        let desktop = platform.desktop();
        let transitions = &self.transitions;
        if !transitions.animations.is_on() || self.display.settling {
            return None;
        }
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
        still_animates(&facts).then_some(from)
    }
}

#[cfg(test)]
#[path = "tests/transitions.rs"]
mod tests;
