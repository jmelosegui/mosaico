//! What the applier does with a report about a window before the tiler
//! sees it: whether it is the echo of one of its own effects, what it ends
//! of the window's motion, and what it shows of what was posted to the
//! window.

use std::time::Instant;

use crate::WindowEvent;

use super::super::Platform;
use super::Applier;
use super::shows::ShowState;

impl Applier {
    /// Why an event is an echo of our own effects, if it is one.
    pub(in crate::tiler) fn echo(
        &mut self,
        event: &WindowEvent,
        platform: &mut dyn Platform,
        now: Instant,
    ) -> Option<&'static str> {
        // Nothing held for a window that no longer exists may be applied.
        if let WindowEvent::Destroyed { window } = *event {
            self.held.forget(window);
            self.release_dropped_focus();
            self.moved_on_screen.remove(&window);
        }
        self.posted_seen(event, platform);
        // A window that slides under a pointer at rest is not one the user
        // pointed at.
        if matches!(event, WindowEvent::MouseHover { .. }) && self.is_moving() {
            return Some("windows are moving");
        }
        let echo = self.expectations.echo(event, now);
        self.motion_seen(event, echo.is_some(), platform.desktop());
        echo
    }

    /// Acts on what a report says of what was posted to a window: one that
    /// reports a show, a hide or a minimize got to it, one that reports where
    /// it is may be where a posted placement took it, one the user grabs is
    /// no longer where it was sent, and one destroyed is forgotten.
    fn posted_seen(&mut self, event: &WindowEvent, platform: &mut dyn Platform) {
        let posted = &mut self.posted;
        match *event {
            WindowEvent::Restored { window } | WindowEvent::Created { window } => {
                posted.reported(window, ShowState::Shown);
            }
            WindowEvent::Hidden { window } => posted.reported(window, ShowState::Hidden),
            WindowEvent::Minimized { window } => posted.reported(window, ShowState::Minimized),
            WindowEvent::LocationChanged { window } => self.landed(window, platform),
            WindowEvent::MoveStarted { window } => {
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
