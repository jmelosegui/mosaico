//! Recognizes the window events our own effects cause.
//!
//! Most effects make the OS report something back, asynchronously and
//! often late: hiding a window can report it hidden, focusing one
//! reports a focus change. Those reports look exactly like the user
//! doing the same thing, so acting on them undoes what the tiler just
//! did. The applier records each effect here as it applies it, and
//! every event is checked against that record before it reaches the
//! tiler. Which reports hiding causes is the platform's to say, in its
//! [`HidingSupport`]. Focus changes are tracked in `echo_focus`.

use std::collections::HashMap;
use std::time::Instant;

use crate::{HidingEvents, HidingSupport, WindowEvent, WindowId};

use super::Effect;
use super::echo_focus::FocusEchoes;

#[path = "echo_hiding.rs"]
mod hiding;

pub use hiding::HIDE_FOCUS_ECHO_TTL;
use hiding::ShowReports;

/// What the applier has done that the OS has yet to report back.
#[derive(Debug, Default)]
pub struct Expectations {
    /// What hiding a window each way reports on this platform.
    hiding: HidingSupport,
    /// Windows hidden by a workspace switch, until shown again, what hiding
    /// them reports, whether the hide was reported, and whether each was
    /// minimized as it was to be shown again.
    hidden: HashMap<WindowId, (HidingEvents, Instant, bool, bool)>,
    /// What the windows a workspace switch showed again are still to
    /// report.
    shown: ShowReports,
    /// Focus changes applied or held back.
    focus: FocusEchoes,
}

impl Expectations {
    /// Expectations on a platform whose ways of hiding report what
    /// `hiding` says.
    pub fn new(hiding: HidingSupport) -> Self {
        Self {
            hiding,
            ..Self::default()
        }
    }

    /// Records an effect the applier has just applied.
    pub fn applied(&mut self, effect: &Effect, now: Instant) {
        match *effect {
            Effect::Hide { window, how } => {
                // A way the platform lacks hid nothing, so reports nothing.
                let events = self.hiding.events(how).unwrap_or_default();
                self.hidden.insert(window, (events, now, false, false));
                self.shown.forget(window);
            }
            Effect::Show { window, .. } => {
                if let Some((events, _, reported, minimized)) = self.hidden.remove(&window) {
                    self.shown.shown(window, events, (reported, minimized), now);
                }
            }
            Effect::Reveal { window } => {
                self.hidden.remove(&window);
            }
            Effect::Focus { window, .. } => self.focus.applied(window, now),
            _ => {}
        }
    }

    /// Records an effect the applier is holding back to apply later.
    pub fn held(&mut self, effect: &Effect) {
        if let Effect::Focus { window, .. } = *effect {
            self.focus.held(window);
        }
    }

    /// Whether a focus change is being held back.
    pub fn is_focus_held(&self) -> bool {
        self.focus.is_held()
    }

    /// Forgets the focus change held back, which the applier dropped and
    /// will not apply.
    pub fn release_held_focus(&mut self) {
        self.focus.release_held();
    }

    /// Checks an event against what we expect, and returns why it is an
    /// echo of our own effects, or `None` when the tiler should see it.
    pub fn echo(&mut self, event: &WindowEvent, now: Instant) -> Option<&'static str> {
        match *event {
            WindowEvent::Destroyed { window } => {
                self.forget(window);
                None
            }
            WindowEvent::Hidden { window } | WindowEvent::Minimized { window } => {
                // Hiding a window a way that reports nothing, such as
                // cloaking, did not cause this: the application did.
                match self.hidden.get_mut(&window) {
                    Some((events, _, reported, _)) if events.reports_hidden => {
                        *reported = true;
                        Some("hidden by a workspace switch")
                    }
                    _ => self.shown.hide_echo(window, now),
                }
            }
            WindowEvent::Restored { window } => self.shown.restore_echo(window, now),
            WindowEvent::Focused { window } => self.focus_echo(window, now),
            _ => None,
        }
    }

    fn focus_echo(&mut self, window: WindowId, now: Instant) -> Option<&'static str> {
        if let Some(reason) = self.focus.echo(window, now) {
            return Some(reason);
        }
        self.hidden
            .get(&window)
            .is_some_and(|&(_, at, ..)| now.duration_since(at) < HIDE_FOCUS_ECHO_TTL)
            .then_some("hidden by a recent workspace switch")
    }

    /// Drops everything recorded about a window.
    pub fn forget(&mut self, window: WindowId) {
        self.hidden.remove(&window);
        self.shown.forget(window);
    }
}

#[cfg(test)]
#[path = "tests/echo_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/echo_hiding_tests.rs"]
mod hiding_tests;

#[cfg(test)]
#[path = "tests/echo_late_tests.rs"]
mod late_tests;
