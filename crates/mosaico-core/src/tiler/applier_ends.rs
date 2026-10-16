//! Effects that reach a window while it moves: what ends its motion, and
//! where the pointer goes.
//!
//! - A window hidden, minimized, destroyed or grabbed by the user stops
//!   where it is, whether the tiler, the user or its application hid or
//!   minimized it. The tiler's own hide or minimize stops it before it is
//!   made, as a frame posted after it would land after it. One shown or
//!   restored later is placed again by the tiler, and that placement snaps.
//! - A window asked to close keeps moving: the application may refuse,
//!   and its destruction or hiding ends the motion.
//! - A window the tiler lets go of is placed at its target, without the
//!   border its motion kept. A display change stops every motion where it
//!   is as soon as it is noted, as `applier_display.rs` says, and the
//!   tiler stopping ends them as `applier_stop.rs` says.
//! - Focusing a moving window with the pointer following, or putting the
//!   pointer on it, aims at the center of its target, not at where the
//!   window is on its way there.

use crate::{Rect, WindowControl, WindowEvent, WindowId};

use super::super::window_effects::report;
use super::super::{Desktop, Effect, Platform};
use super::Applier;

impl Applier {
    /// Stops the window's motion where the window is, and forgets the
    /// target a display change kept for it. It is not placed at its target,
    /// and its border is left to what the tiler decides next.
    pub(super) fn stop_motion(&mut self, window: WindowId) {
        self.transitions.motions.end(window);
        self.forget_kept_border(window);
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
    /// full, then drawing its border there. Does nothing for a window that
    /// is not moving.
    pub(super) fn finish_motion(&mut self, window: WindowId, platform: &mut dyn Platform) {
        if let Some(target) = self.transitions.motions.end(window) {
            self.in_place(window, target, platform);
        }
    }

    /// Places a moving window the tiler lets go of at its target, in full,
    /// and forgets the border its motion kept: the step that lets go of the
    /// window drops its border, so drawing it first would be wasted. Does
    /// nothing for a window that is not moving.
    pub(super) fn let_go(&mut self, window: WindowId, platform: &mut dyn Platform) {
        if let Some(target) = self.transitions.motions.end(window) {
            self.forget_kept_border(window);
            self.send_to(window, target, platform);
        }
    }

    /// Places a window whose motion is over at `target`, in full, and
    /// draws the border it kept for the motion around it. The platform
    /// orders the placement behind the frames posted to the window before,
    /// so the border is drawn at once only when the window is there
    /// already, and otherwise once it lands (`applier_landing.rs`).
    pub(super) fn in_place(&mut self, window: WindowId, target: Rect, platform: &mut dyn Platform) {
        if self.send_to(window, target, platform) {
            let border = self.take_kept_border(window);
            self.defer_border(window, border);
        } else {
            self.draw_kept_border(window, target, platform.decorations());
        }
    }

    /// Whether `window` is moving to a target.
    pub(super) fn is_moving_window(&self, window: WindowId) -> bool {
        self.transitions.motions.is_moving(window)
    }

    /// Where the pointer goes for `window`, if not where the desktop has
    /// it: the target of its motion, or where it was last sent while that
    /// may still be on its way.
    pub(super) fn aim(&self, window: WindowId, control: &dyn WindowControl) -> Option<Rect> {
        let target = self.transitions.motions.target(window);
        target.or_else(|| self.last_sent(window, control))
    }

    /// Carries out `effect`, a focus change with the pointer following or
    /// a move of the pointer onto `window`, which is moving or has what it
    /// was last sent on its way: the pointer goes to the center of where
    /// the window is going.
    pub(super) fn point_at_target(
        &self,
        effect: &Effect,
        window: WindowId,
        control: &dyn WindowControl,
    ) {
        if let Effect::Focus { .. } = effect {
            report("focus", window, control.focus(window));
        }
        if let Some(target) = self.aim(window, control) {
            let (x, y) = (target.center_x(), target.center_y());
            report("center the pointer on", window, control.move_cursor(x, y));
        }
    }
}
