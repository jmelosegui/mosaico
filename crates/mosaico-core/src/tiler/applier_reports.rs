//! What the applier does with a report about a window before the tiler
//! sees it: whether it is the echo of one of its own effects, what it ends
//! of the window's motion, and what it shows of what was posted to the
//! window.

use std::time::Instant;

use crate::{WindowEvent, WindowId};

use super::super::Platform;
use super::Applier;
use super::shows::ShowState;

impl Applier {
    /// Why an event is an echo of our own effects, or is not for the tiler
    /// to see, if it is either, with the tiler's focus on `focused`.
    pub(in crate::tiler) fn echo(
        &mut self,
        event: &WindowEvent,
        focused: Option<WindowId>,
        platform: &mut dyn Platform,
        now: Instant,
    ) -> Option<&'static str> {
        // Nothing held for a window that no longer exists may be applied,
        // nor what was held for one the user grabs, which was decided
        // before they took it.
        if let WindowEvent::Destroyed { window } | WindowEvent::MoveStarted { window, .. } = *event
        {
            self.held.forget(window);
            self.release_dropped_focus();
        }
        if let WindowEvent::Destroyed { window } = *event {
            self.moved_on_screen.remove(&window);
        }
        self.posted_seen(event, platform, now);
        // A window that slides under a pointer at rest is not one the user
        // pointed at, and focus follows mouse rests while the user holds a
        // window (`applier_drag.rs`).
        if let WindowEvent::MouseHover { .. } = event {
            if self.is_moving() {
                return Some("windows are moving");
            }
            if self.holds_pointer() {
                return Some("a window is held");
            }
        }
        let echo = self.expectations.echo(event, now);
        self.drag_seen(event, echo.is_some(), focused, platform.desktop());
        self.motion_seen(event, echo.is_some(), platform.desktop(), now);
        echo
    }

    /// Acts on what a report at `now` says of what was posted to a window:
    /// one that reports a show, a hide or a minimize got to it, one that
    /// reports where it is may be where a posted placement took it, one the
    /// user grabs is no longer where it was sent, and one destroyed is
    /// forgotten.
    fn posted_seen(&mut self, event: &WindowEvent, platform: &mut dyn Platform, now: Instant) {
        let posted = &mut self.posted;
        match *event {
            WindowEvent::Restored { window } | WindowEvent::Created { window } => {
                posted.reported(window, ShowState::Shown);
            }
            WindowEvent::Hidden { window } => posted.reported(window, ShowState::Hidden),
            WindowEvent::Minimized { window } => posted.reported(window, ShowState::Minimized),
            WindowEvent::LocationChanged { window } => self.landed(window, platform, now),
            WindowEvent::MoveStarted { window, .. } => {
                posted.sent.remove(&window);
                posted.placed.remove(&window);
            }
            WindowEvent::Destroyed { window } => {
                posted.sent.remove(&window);
                posted.placed.remove(&window);
                posted.states.remove(&window);
                posted.framed.remove(&window);
            }
            _ => {}
        }
    }
}
