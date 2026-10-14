//! What one daemon hid that the next one has to show again.
//!
//! A workspace switch hides windows, and only a clean shutdown shows
//! them again. [`HiddenWindows`] is the record a platform keeps of those
//! windows while it applies effects, so that a daemon started after one
//! that was killed can find them and bring them back.

use crate::WindowId;

use super::Effect;

/// What identifies a window beyond its id, which the platform may reuse
/// once the window is gone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowIdentity {
    /// Process that owns the window.
    pub process: u32,
    /// Window class or the platform's nearest equivalent.
    pub class: String,
}

/// Windows hidden by workspace switching, as a daemon leaves them.
///
/// Records nothing yet: a daemon does not remember across runs which
/// windows it hid (#67).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HiddenWindows;

impl HiddenWindows {
    /// Updates the record for an effect the platform just applied.
    /// `identity` looks up the window the effect is about. Returns
    /// whether the record changed.
    pub fn applied(
        &mut self,
        _effect: &Effect,
        _identity: impl FnOnce(WindowId) -> Option<WindowIdentity>,
    ) -> bool {
        false
    }

    /// Drops a window that no longer exists. Returns whether the record
    /// changed.
    pub fn forget(&mut self, _window: WindowId) -> bool {
        false
    }

    /// Whether nothing is recorded.
    pub fn is_empty(&self) -> bool {
        true
    }

    /// The effects that show again what a previous daemon hid.
    pub fn restore(&self, _identity: impl Fn(WindowId) -> Option<WindowIdentity>) -> Vec<Effect> {
        Vec::new()
    }

    /// The record as text, to keep it across daemon runs.
    pub fn to_text(&self) -> String {
        String::new()
    }

    /// Reads a record written by [`HiddenWindows::to_text`].
    pub fn from_text(_text: &str) -> Result<Self, String> {
        Ok(Self)
    }
}
