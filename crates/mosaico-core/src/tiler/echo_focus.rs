//! Recognizes the focus reports our own focus changes cause.
//!
//! The OS reports a focus change the applier made late, often after the
//! user has asked for the next one, and it makes changes of its own
//! while one of ours is still held back. Following either report would
//! move focus somewhere the user did not ask for.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::WindowId;

/// How long the OS may take to report a focus change we made.
///
/// Focus reports routinely arrive hundreds of milliseconds after the
/// change, and later still under load.
pub const FOCUS_ECHO_TTL: Duration = Duration::from_millis(1500);

/// How many recent focus changes are remembered.
const FOCUS_INTENTS: usize = 8;

/// Focus changes the applier has made or is holding back.
#[derive(Debug, Default)]
pub(super) struct FocusEchoes {
    /// Focus change held back by the applier, not applied yet.
    held: Option<WindowId>,
    /// Focus changes applied, oldest first.
    applied: VecDeque<(WindowId, Instant)>,
}

impl FocusEchoes {
    /// Records a focus change the applier has just made.
    pub(super) fn applied(&mut self, window: WindowId, now: Instant) {
        if self.held == Some(window) {
            self.held = None;
        }
        self.applied
            .retain(|&(_, at)| now.duration_since(at) < FOCUS_ECHO_TTL);
        self.applied.push_back((window, now));
        while self.applied.len() > FOCUS_INTENTS {
            self.applied.pop_front();
        }
    }

    /// Records a focus change the applier is holding back.
    pub(super) fn held(&mut self, window: WindowId) {
        self.held = Some(window);
    }

    /// Whether a focus change is being held back.
    pub(super) fn is_held(&self) -> bool {
        self.held.is_some()
    }

    /// Why a report that `window` got focus is an echo of our own focus
    /// changes, or `None` when it is not one.
    pub(super) fn echo(&mut self, window: WindowId, now: Instant) -> Option<&'static str> {
        self.applied
            .retain(|&(_, at)| now.duration_since(at) < FOCUS_ECHO_TTL);
        let latest = self.applied.back().map(|&(w, _)| w);
        if latest != Some(window) && self.applied.iter().any(|&(w, _)| w == window) {
            return Some("stale report of an earlier focus change");
        }
        // The OS can make its own focus change while ours is held back,
        // typically handing focus to whatever sits behind a window that
        // closed, often on another monitor. Following it would flash the
        // border there before ours lands. The held change always lands
        // on the next flush, so this cannot get stuck.
        if self.held.is_some_and(|held| held != window) {
            return Some("another window is about to take focus");
        }
        None
    }
}
