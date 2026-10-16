//! Ending motions at once: as the tiler stops, as transitions turn off,
//! and as posting frames fails. A window whose motion ends this way is
//! placed at its target in full, which the platform orders behind the
//! frames posted to it before. While transitions are off, the platform is
//! asked to forget the windows it posts to as soon as it can, so that
//! changes to them are made at once again.

use std::time::Instant;

use crate::Error;
use crate::motion::{Ending, Kept, Tick};

use super::super::{Desktop, Platform};
use super::Applier;

impl Applier {
    /// Ends every motion at its target as the tiler stops, at `now`, before
    /// any window is revealed, and places the windows a display change
    /// stopped at theirs. No border is drawn: the tiler hides them all.
    pub(in crate::tiler) fn finish_motions(&mut self, platform: &mut dyn Platform, now: Instant) {
        self.transitions.borders.clear();
        let stopped = self.take_stopped();
        for (window, target) in self.transitions.motions.clear() {
            let placed = self.send_to(window, target, platform);
            self.log_end(window, Ending::Shutdown, Some(placed), now);
        }
        for (window, target) in stopped {
            let placed = self.send_to(window, target, platform);
            self.log_kept(window, Kept::Shutdown, placed, now);
        }
    }

    /// Turns transitions off for the rest of the session, at `now`, as on a
    /// platform that cannot move windows frame by frame: nothing animates
    /// from then on, and every moving window is placed at its target, with
    /// the border it kept for its motion drawn there. Then the platform
    /// forgets the windows it no longer needs to post to.
    pub(in crate::tiler) fn turn_off_transitions(
        &mut self,
        platform: &mut dyn Platform,
        now: Instant,
    ) {
        self.transitions.frames_supported = false;
        for (window, target) in self.transitions.motions.clear() {
            let placed = self.in_place(window, target, platform);
            self.log_end(window, Ending::TransitionsOff, Some(placed), now);
        }
        self.settle_while_off(platform);
    }

    /// Whether windows can still be moved frame by frame this session: not
    /// once transitions were turned off for it.
    pub(in crate::tiler) fn has_frames(&self) -> bool {
        self.transitions.frames_supported
    }

    /// Has the platform forget the windows given frames that it no longer
    /// needs to post changes to, while transitions are off: when they turn
    /// off, and at the start of every step and flush after, before the
    /// tiler reads the desktop, for as long as one of them is still posted
    /// to, as within the grace or while its application does not respond.
    /// They are off while the config, or the system's setting it follows,
    /// has them off, or the platform cannot move windows frame by frame.
    pub(in crate::tiler) fn settle_while_off(&mut self, platform: &dyn Platform) {
        if self.posted.framed.is_empty() || self.transitions_on(platform.desktop()) {
            return;
        }
        let control = platform.control();
        control.settle_responding();
        self.posted
            .framed
            .retain(|&window| control.posts_behind(window));
    }

    /// Whether placements may animate on `desktop`, as far as the config,
    /// the system's setting and the platform go.
    fn transitions_on(&self, desktop: &dyn Desktop) -> bool {
        let transitions = &self.transitions;
        let animations = &transitions.animations;
        animations.is_on()
            && transitions.frames_supported
            && (!animations.follow_system || desktop.animations_allowed())
    }

    /// Acts on `place_frames` failing at `now` for the frames of `tick`. A
    /// platform that cannot move windows frame by frame turns transitions
    /// off, logged once. Any other failure is logged once until frames work
    /// again, and the windows the frames were for are placed at their
    /// targets, as for placements that snap. Transitions stay on.
    pub(super) fn frames_failed(
        &mut self,
        error: &Error,
        tick: &Tick,
        platform: &mut dyn Platform,
        now: Instant,
    ) {
        if let Error::Unsupported(_) = error {
            crate::log_info!("Transitions are off: windows cannot move frame by frame ({error})");
            self.turn_off_transitions(platform, now);
            return;
        }
        if !std::mem::replace(&mut self.transitions.frames_failing, true) {
            crate::log_warn!("Windows went straight to their places: {error}");
        }
        for frame in &tick.in_flight {
            self.finish_motion(frame.window, Ending::FramesFailed, platform, now);
        }
    }
}
