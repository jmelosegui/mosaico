//! The windows a posted placement is taking somewhere, until they land.
//!
//! A placement posted behind what is on its way to a window lands once the
//! window gets to it, and the repaint posted with it only repairs what was
//! wrong before the changes queued ahead of it. So the window is repainted
//! again once it lands, and the border its motion kept, or that the tiler
//! decided for it meanwhile, is drawn then, around where it was placed:
//! drawn before, it would run ahead of a window that fell behind, or be
//! left where the window was.
//!
//! A window lands once it reports being there, no edge of it more than
//! [`MIN_TRAVEL`](crate::motion::MIN_TRAVEL) pixels from where it was
//! placed, as a window that rounds its own size may be; once it reports
//! being anywhere while nothing sent to it may still be on its way; and
//! [`LANDING_WAIT`] after the placement, if it has moved since or is there
//! by then, as it reports nothing for a placement that moves nothing.
//! One that has not moved since, as its application has not got to the
//! placement, is no longer waited for by time: it lands on its next
//! report, which comes once its application gets there, and until then
//! its border stays where the window was last found. A placement that
//! finds the window where the last thing sent to it took it, and there,
//! is not waited for at all (`applier_posted.rs`).

use std::time::{Duration, Instant};

use crate::frames::PENDING_LIMIT;
use crate::motion::{Landed, hardly_apart};
use crate::{Rect, WindowId};

use super::super::window_effects::report;
use super::super::{BorderStyle, Desktop, Platform};
use super::Applier;
use super::borders::draw_border;

/// How long after a posted placement a window that has not reported being
/// there, but has moved since or is there, is repainted again, and has its
/// border drawn there, anyway. A window that keeps away from where it was
/// placed, as one sized to whole text cells does, may report its last move
/// before the placement lands, and the placement then reports nothing;
/// and a report may be missed. Twice [`PENDING_LIMIT`]: an application
/// that keeps up gets to its placement within a few frames, and one that
/// does not move is given a frame at most once in that time, so it has at
/// most one queued ahead of the placement, and has got to both by then if
/// it takes up to that for each. A border that comes half a second late is
/// seen to come late, not taken for one that is missing.
pub(super) const LANDING_WAIT: Duration = PENDING_LIMIT.saturating_mul(2);

/// A posted placement on its way to its window.
#[derive(Debug)]
pub(super) struct Landing {
    /// Where it takes the window.
    pub(super) rect: Rect,
    /// Where the desktop had the window as the placement was posted.
    pub(super) from: Option<Rect>,
    /// The border to draw there once the window lands.
    pub(super) border: Option<BorderStyle>,
    /// When the wait by time ends, or `None` once it ended with the window
    /// where it was: it then lands on its next report.
    pub(super) until: Option<Instant>,
    /// When it was posted, if it ended a transition: the log hears it land.
    pub(super) logged: Option<Instant>,
}

impl Applier {
    /// Lands `window`, which reports at `now` where it is, if a posted
    /// placement is taking it somewhere and it is there, nothing sent to it
    /// may still be on its way, or it is no longer waited for by time.
    pub(super) fn landed(&mut self, window: WindowId, platform: &mut dyn Platform, now: Instant) {
        let Some(landing) = self.posted.placed.get(&window) else {
            return;
        };
        let there = is_there(platform.desktop(), window, landing.rect);
        let late = landing.until.is_none();
        if there || late || !platform.control().posts_behind(window) {
            self.land(window, platform, Landed::reported(there, late), now);
        }
    }

    /// Ends each wait by time due by `now`, [`LANDING_WAIT`] after its
    /// placement, in the order of their placements: a window there, or that
    /// moved since the placement was posted, lands, and one that did not is
    /// left to land on its next report.
    pub(super) fn land_overdue(&mut self, platform: &mut dyn Platform, now: Instant) {
        let mut overdue: Vec<(Instant, WindowId)> = (self.posted.placed.iter())
            .filter_map(|(&window, landing)| landing.until.map(|until| (until, window)))
            .filter(|&(until, _)| until <= now)
            .collect();
        overdue.sort_unstable();
        for (_, window) in overdue {
            let desktop = platform.desktop();
            let Some(landing) = self.posted.placed.get_mut(&window) else {
                continue;
            };
            let moved = desktop.rect(window) != landing.from;
            if moved || is_there(desktop, window, landing.rect) {
                self.land(window, platform, Landed::Overdue, now);
            } else {
                landing.until = None;
            }
        }
    }

    /// When the first wait by time ends, if one is on.
    pub(super) fn landing_due(&self) -> Option<Instant> {
        let landings = self.posted.placed.values();
        landings.filter_map(|landing| landing.until).min()
    }

    /// Whether `window` is still to land where a posted placement takes it.
    pub(super) fn is_landing(&self, window: WindowId) -> bool {
        self.posted.placed.contains_key(&window)
    }

    /// Keeps `border` to draw once `window` lands, if it is still to.
    pub(super) fn defer_border(&mut self, window: WindowId, border: Option<BorderStyle>) {
        if let Some(landing) = self.posted.placed.get_mut(&window) {
            landing.border = border;
        }
    }

    /// Forgets the border kept to draw once `window` lands: the tiler
    /// dropped it, or the window leaves the view.
    pub(super) fn forget_deferred_border(&mut self, window: WindowId) {
        self.defer_border(window, None);
    }

    /// Stops waiting for `window` to land, and forgets what was kept for
    /// it: it moves on, and what comes next is waited for instead.
    pub(super) fn forget_landing(&mut self, window: WindowId) {
        self.posted.placed.remove(&window);
    }

    /// Ends the wait for `window` at `now`, as it landed `how`: repaints it
    /// again, and draws the border kept for it around where it was placed.
    fn land(&mut self, window: WindowId, platform: &mut dyn Platform, how: Landed, now: Instant) {
        if let Some(landing) = self.posted.placed.remove(&window) {
            report("repaint", window, platform.control().repaint(window));
            draw_border(window, landing.border, landing.rect, platform.decorations());
            self.log_landed(window, landing.logged, how, now);
        }
    }
}

/// Whether `desktop` has `window` at `rect`, no edge of it more than
/// [`MIN_TRAVEL`](crate::motion::MIN_TRAVEL) pixels away.
pub(super) fn is_there(desktop: &dyn Desktop, window: WindowId, rect: Rect) -> bool {
    desktop
        .rect(window)
        .is_some_and(|at| hardly_apart(at, rect))
}
