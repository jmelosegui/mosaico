//! Recognizes the reports that hiding and showing windows for a workspace
//! switch cause after the fact: focus reports for a window just hidden,
//! and the reports of a hide or a show that land late.
//!
//! A hide or a show posted behind a frame on its way to the window happens
//! when the application gets to it, which can be well after the tiler made
//! it: a show can wait behind a hide, which is then reported after the
//! tiler showed the window again. Taking such a report for the user's would
//! take the window out of the layout, or put it back and focus it.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::frames::ASYNC_GRACE;
use crate::{HidingEvents, WindowId};

use super::Expectations;

/// How long after hiding a window a focus report for it is taken as
/// ours rather than the user's.
///
/// Hiding the window that held focus makes the OS report focus changes
/// for it after the switch. Acting on one would switch straight back to
/// the workspace it is on, and the switch would cause another. This is
/// long enough to absorb those reports without being noticeable when
/// the user really does pick a hidden window, from the taskbar say.
pub const HIDE_FOCUS_ECHO_TTL: Duration = Duration::from_millis(500);

/// The reports still to come for the windows a workspace switch showed
/// again: their restore, where showing reports one, and the hide before
/// the show, where it was not reported yet. Each is expected until it
/// comes, until the window is hidden or shown again or destroyed, and for
/// at most [`ASYNC_GRACE`] after the show, as long as a change the platform
/// posts behind frames may take to land.
#[derive(Debug, Default)]
pub(super) struct ShowReports {
    /// When each window whose hide is still to be reported was shown.
    hides: HashMap<WindowId, Instant>,
    /// When each window whose restore is still to be reported was shown.
    restores: HashMap<WindowId, Instant>,
}

impl ShowReports {
    /// Notes that `window` was shown again at `now`, after a hide that
    /// reports what `events` says, and was `reported` already or not, as the
    /// window was `minimized` or not. A hide still to be reported is
    /// expected late. So is a restore, where showing reports one, if the
    /// show restores the window: one its hide is still to land on, or one
    /// minimized, and not one the user restored already, as from the
    /// taskbar.
    pub(super) fn shown(
        &mut self,
        window: WindowId,
        events: HidingEvents,
        (reported, minimized): (bool, bool),
        now: Instant,
    ) {
        self.forget(window);
        if events.reports_hidden && !reported {
            self.hides.insert(window, now);
        }
        if events.reports_restored && (!reported || minimized) {
            self.restores.insert(window, now);
        }
    }

    /// Why a report at `now` that `window` was hidden or minimized is the
    /// late report of a hide of ours.
    pub(super) fn hide_echo(&mut self, window: WindowId, now: Instant) -> Option<&'static str> {
        let shown = self.hides.remove(&window)?;
        in_time(shown, now).then_some("hidden by a workspace switch, late")
    }

    /// Why a report at `now` that `window` was restored is the report of a
    /// show of ours.
    pub(super) fn restore_echo(&mut self, window: WindowId, now: Instant) -> Option<&'static str> {
        let shown = self.restores.remove(&window)?;
        in_time(shown, now).then_some("restored by a workspace switch")
    }

    /// Whether the hide before the show of `window` is still expected to be
    /// reported at `now`.
    fn hide_expected(&self, window: WindowId, now: Instant) -> bool {
        self.hides
            .get(&window)
            .is_some_and(|&shown| in_time(shown, now))
    }

    /// Forgets what `window` was to report, as when it is hidden again or
    /// destroyed.
    pub(super) fn forget(&mut self, window: WindowId) {
        self.hides.remove(&window);
        self.restores.remove(&window);
    }
}

/// Whether a report at `now` can still be one of a show made at `shown`.
fn in_time(shown: Instant, now: Instant) -> bool {
    now.duration_since(shown) < ASYNC_GRACE
}

impl Expectations {
    /// Notes whether `window` is minimized as it is about to be shown
    /// again: only then, or while its hide is still to be reported, does
    /// the show restore it.
    pub fn showing(&mut self, window: WindowId, minimized: bool) {
        if let Some(hidden) = self.hidden.get_mut(&window) {
            hidden.3 = minimized;
        }
    }

    /// Whether `window` was hidden a way that reports it, and the report
    /// is still to come.
    pub fn hide_unreported(&self, window: WindowId) -> bool {
        let unreported = |&(events, _, reported, _): &(HidingEvents, _, bool, bool)| {
            events.reports_hidden && !reported
        };
        self.hidden.get(&window).is_some_and(unreported)
    }

    /// Whether `window`, which a workspace switch showed again, is still
    /// expected at `now` to report the hide before the show, late.
    pub fn hide_expected(&self, window: WindowId, now: Instant) -> bool {
        self.shown.hide_expected(window, now)
    }
}
