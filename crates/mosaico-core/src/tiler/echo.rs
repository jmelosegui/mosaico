//! Recognizes the window events our own effects cause.
//!
//! Most effects make the OS report something back, asynchronously and
//! often late: hiding a window reports it hidden, focusing one reports
//! a focus change. Those reports look exactly like the user doing the
//! same thing, so acting on them undoes what the tiler just did. The
//! applier records each effect here as it applies it, and every event
//! is checked against that record before it reaches the tiler. Focus
//! changes are tracked in `echo_focus`.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::config::HidingBehaviour;
use crate::{WindowEvent, WindowId};

use super::Effect;
use super::echo_focus::{FOCUS_ECHO_TTL, FocusEchoes};

/// How long after hiding a window a focus report for it is taken as
/// ours rather than the user's.
///
/// Hiding the window that held focus makes the OS report focus changes
/// for it after the switch. Acting on one would switch straight back to
/// the workspace it is on, and the switch would cause another. This is
/// long enough to absorb those reports without being noticeable when
/// the user really does pick a hidden window, from the taskbar say.
pub const HIDE_FOCUS_ECHO_TTL: Duration = Duration::from_millis(500);

/// What the applier has done that the OS has yet to report back.
#[derive(Debug, Default)]
pub struct Expectations {
    /// Windows hidden by a workspace switch, until shown again.
    hidden: HashMap<WindowId, (HidingBehaviour, Instant)>,
    /// Windows restored from minimized by a workspace switch.
    restored: HashMap<WindowId, Instant>,
    /// Focus changes applied or held back.
    focus: FocusEchoes,
}

impl Expectations {
    /// Records an effect the applier has just applied.
    pub fn applied(&mut self, effect: &Effect, now: Instant) {
        match *effect {
            Effect::Hide { window, how } => {
                self.hidden.insert(window, (how, now));
                self.restored.remove(&window);
            }
            Effect::Show { window, .. } => {
                // Only a window we minimized reports being restored.
                if let Some((HidingBehaviour::Minimize, _)) = self.hidden.remove(&window) {
                    self.restored.insert(window, now);
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

    /// Checks an event against what we expect, and returns why it is an
    /// echo of our own effects, or `None` when the tiler should see it.
    pub fn echo(&mut self, event: &WindowEvent, now: Instant) -> Option<&'static str> {
        match *event {
            WindowEvent::Destroyed { window } => {
                self.forget(window);
                None
            }
            WindowEvent::Hidden { window } | WindowEvent::Minimized { window } => {
                // Cloaking reports nothing, so a hide report for a
                // cloaked window comes from the application itself.
                self.hidden
                    .get(&window)
                    .filter(|(how, _)| *how != HidingBehaviour::Cloak)
                    .map(|_| "hidden by a workspace switch")
            }
            WindowEvent::Restored { window } => {
                let at = self.restored.remove(&window)?;
                (now.duration_since(at) < FOCUS_ECHO_TTL)
                    .then_some("restored by a workspace switch")
            }
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
            .is_some_and(|&(_, at)| now.duration_since(at) < HIDE_FOCUS_ECHO_TTL)
            .then_some("hidden by a recent workspace switch")
    }

    /// Drops everything recorded about a window.
    pub fn forget(&mut self, window: WindowId) {
        self.hidden.remove(&window);
        self.restored.remove(&window);
    }
}

#[cfg(test)]
#[path = "tests/echo_tests.rs"]
mod tests;
