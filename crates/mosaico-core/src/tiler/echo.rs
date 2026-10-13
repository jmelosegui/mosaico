//! Recognizes the window events our own effects cause.
//!
//! Most effects make the OS report something back, asynchronously and
//! often late: hiding a window reports it hidden, focusing one reports
//! a focus change. Those reports look exactly like the user doing the
//! same thing, so acting on them undoes what the tiler just did. The
//! applier records each effect here as it applies it, and every event
//! is checked against that record before it reaches the tiler.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use crate::config::HidingBehaviour;
use crate::{WindowEvent, WindowId};

use super::Effect;

/// How long the OS may take to report a focus change we made.
///
/// Focus reports routinely arrive hundreds of milliseconds after the
/// change, and later still under load.
pub const FOCUS_ECHO_TTL: Duration = Duration::from_millis(1500);

/// How long after hiding a window a focus report for it is taken as
/// ours rather than the user's.
///
/// Hiding the window that held focus makes the OS report focus changes
/// for it after the switch. Acting on one would switch straight back to
/// the workspace it is on, and the switch would cause another. This is
/// long enough to absorb those reports without being noticeable when
/// the user really does pick a hidden window, from the taskbar say.
pub const HIDE_FOCUS_ECHO_TTL: Duration = Duration::from_millis(500);

/// How many recent focus changes are remembered.
const FOCUS_INTENTS: usize = 8;

/// What the applier has done that the OS has yet to report back.
#[derive(Debug, Default)]
pub struct Expectations {
    /// Windows hidden by a workspace switch, until shown again.
    hidden: HashMap<WindowId, (HidingBehaviour, Instant)>,
    /// Windows restored from minimized by a workspace switch.
    restored: HashMap<WindowId, Instant>,
    /// Focus change held back by the applier, not applied yet.
    held_focus: Option<WindowId>,
    /// Focus changes applied, oldest first.
    focused: VecDeque<(WindowId, Instant)>,
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
            Effect::Focus { window, .. } => {
                if self.held_focus == Some(window) {
                    self.held_focus = None;
                }
                self.focused
                    .retain(|&(_, at)| now.duration_since(at) < FOCUS_ECHO_TTL);
                self.focused.push_back((window, now));
                while self.focused.len() > FOCUS_INTENTS {
                    self.focused.pop_front();
                }
            }
            _ => {}
        }
    }

    /// Records an effect the applier is holding back to apply later.
    pub fn held(&mut self, effect: &Effect) {
        if let Effect::Focus { window, .. } = *effect {
            self.held_focus = Some(window);
        }
    }

    /// Whether a focus change is being held back.
    pub fn is_focus_held(&self) -> bool {
        self.held_focus.is_some()
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
        self.focused
            .retain(|&(_, at)| now.duration_since(at) < FOCUS_ECHO_TTL);
        let latest = self.focused.back().map(|&(w, _)| w);
        if latest != Some(window) && self.focused.iter().any(|&(w, _)| w == window) {
            return Some("stale report of an earlier focus change");
        }
        // The OS can make its own focus change while ours is held back,
        // typically handing focus to whatever sits behind a window that
        // closed, often on another monitor. Following it would flash the
        // border there before ours lands. The held change always lands
        // on the next flush, so this cannot get stuck.
        if self.held_focus.is_some_and(|held| held != window) {
            return Some("another window is about to take focus");
        }
        if self
            .hidden
            .get(&window)
            .is_some_and(|&(_, at)| now.duration_since(at) < HIDE_FOCUS_ECHO_TTL)
        {
            return Some("hidden by a recent workspace switch");
        }
        None
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
