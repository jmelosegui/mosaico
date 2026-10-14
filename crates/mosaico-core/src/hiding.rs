//! The ways a platform can hide windows, and what each one reports.
//!
//! A workspace switch hides the windows of the workspace left, in one of
//! the ways [`HidingBehaviour`](crate::config::HidingBehaviour) names.
//! Platforms differ in which of those ways they have, and in which
//! window events hiding a window and showing it again cause. The tiler
//! needs both: it must not hide windows in a way the platform lacks, and
//! it must tell the events its own hiding causes apart from the user's.
//! [`HidingSupport`] holds the platform's answers, taken once from
//! [`WindowControl::hiding_events`].

use crate::config::HidingBehaviour;

/// The window events a platform reports when it hides a window one way,
/// and when it shows the window again. The default reports nothing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HidingEvents {
    /// Hiding the window reports it hidden or minimized, as
    /// [`WindowEvent::Hidden`](crate::WindowEvent::Hidden) or
    /// [`WindowEvent::Minimized`](crate::WindowEvent::Minimized).
    pub reports_hidden: bool,
    /// Showing it again reports it restored, as
    /// [`WindowEvent::Restored`](crate::WindowEvent::Restored).
    pub reports_restored: bool,
}

/// Which ways of hiding windows a platform has, and what each reports.
///
/// The default has every way, each reporting what it usually does:
/// cloaking reports nothing, hiding reports the window hidden, and
/// minimizing reports it minimized and, once it is shown again,
/// restored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HidingSupport {
    cloak: Option<HidingEvents>,
    hide: Option<HidingEvents>,
    minimize: Option<HidingEvents>,
}

impl HidingSupport {
    /// Asks `events_of` about every way of hiding. A way it answers
    /// `None` for is one the platform does not have.
    pub fn new(mut events_of: impl FnMut(HidingBehaviour) -> Option<HidingEvents>) -> Self {
        Self {
            cloak: events_of(HidingBehaviour::Cloak),
            hide: events_of(HidingBehaviour::Hide),
            minimize: events_of(HidingBehaviour::Minimize),
        }
    }

    /// What hiding a window `how` reports, or `None` when the platform
    /// cannot hide windows that way.
    pub fn events(&self, how: HidingBehaviour) -> Option<HidingEvents> {
        match how {
            HidingBehaviour::Cloak => self.cloak,
            HidingBehaviour::Hide => self.hide,
            HidingBehaviour::Minimize => self.minimize,
        }
    }
}

impl Default for HidingSupport {
    fn default() -> Self {
        Self::new(|how| {
            Some(HidingEvents {
                reports_hidden: how != HidingBehaviour::Cloak,
                reports_restored: how == HidingBehaviour::Minimize,
            })
        })
    }
}

#[cfg(test)]
#[path = "hiding_tests.rs"]
mod tests;
