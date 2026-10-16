//! Effects that reach a window while it moves: what ends its motion, and
//! where the pointer goes.
//!
//! - A window hidden, minimized, destroyed or grabbed by the user stops
//!   where it is, whether the tiler, the user or its application hid or
//!   minimized it. One shown or restored later is placed again by the
//!   tiler, and that placement snaps.
//! - A window asked to close keeps moving: the application may refuse,
//!   and its destruction or hiding ends the motion.
//! - A window the tiler lets go of and the tiler stopping finish the
//!   motion at its target. A display change stops every motion where it
//!   is as soon as it is noted, as `applier_display.rs` says.
//! - Focusing a moving window with the pointer following, or putting the
//!   pointer on it, aims at the center of its target, not at where the
//!   window is on its way there.

use crate::{WindowControl, WindowEvent, WindowId};

use super::super::window_effects::{place_and_repaint, report};
use super::super::{Desktop, Effect, Platform};
use super::Applier;

impl Applier {
    /// Stops the window's motion where the window is, and forgets the
    /// target a display change kept for it. It is not placed at its target.
    pub(super) fn stop_motion(&mut self, window: WindowId) {
        self.transitions.motions.end(window);
        self.forget_stopped(window);
    }

    /// Ends the motion of the window `event` is about, for what the user,
    /// its application or the system did to it: one destroyed, grabbed by
    /// the user, or minimized or hidden other than by the tiler stops
    /// where it is, so the `Untile` that takes it out of the layout has
    /// nothing to finish. A report counts only when `desktop` agrees: a
    /// window that stays shown, as an application recreating its surface
    /// reports, goes on. The tiler's own hide or minimize, which `echo`
    /// says the event is, stopped it already.
    pub(super) fn motion_seen(&mut self, event: &WindowEvent, echo: bool, desktop: &dyn Desktop) {
        match *event {
            WindowEvent::Destroyed { window } | WindowEvent::MoveStarted { window } => {
                self.stop_motion(window);
            }
            WindowEvent::Hidden { window } if !echo && !desktop.is_visible(window) => {
                self.stop_motion(window);
            }
            WindowEvent::Minimized { window } if !echo && desktop.is_minimized(window) => {
                self.stop_motion(window);
            }
            _ => {}
        }
    }

    /// Finishes the window's motion at its target, placing it there in
    /// full. Does nothing for a window that is not moving.
    pub(super) fn finish_motion(&mut self, window: WindowId, platform: &mut dyn Platform) {
        if let Some(target) = self.transitions.motions.end(window) {
            place_and_repaint(window, target, platform.control());
        }
    }

    /// Finishes every motion at its target, as the tiler stopping does,
    /// placing each window there in full, the ones a display change stopped
    /// too.
    pub(in crate::tiler) fn finish_motions(&mut self, platform: &mut dyn Platform) {
        let stopped = self.take_stopped();
        for (window, target) in self.transitions.motions.clear().into_iter().chain(stopped) {
            place_and_repaint(window, target, platform.control());
        }
    }

    /// Whether `window` is moving to a target.
    pub(super) fn is_moving_window(&self, window: WindowId) -> bool {
        self.transitions.motions.is_moving(window)
    }

    /// Carries out `effect`, a focus change with the pointer following or
    /// a move of the pointer onto `window`, which is moving: the pointer
    /// goes to the center of its target.
    pub(super) fn point_at_target(
        &self,
        effect: &Effect,
        window: WindowId,
        control: &dyn WindowControl,
    ) {
        if let Effect::Focus { .. } = effect {
            report("focus", window, control.focus(window));
        }
        if let Some(target) = self.transitions.motions.target(window) {
            let (x, y) = (target.center_x(), target.center_y());
            report("center the pointer on", window, control.move_cursor(x, y));
        }
    }
}
