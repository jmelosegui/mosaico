//! How the fake desktop moves its windows to the frames of transitions,
//! and keeps their other changes in order with them.
//!
//! The fake decides about frames with the [`FrameTable`] every platform
//! uses, through a probe over its windows, on the platform's clock. A
//! frame lands at once, unless the window's `frame_delay` says its
//! application is slow: then it is queued, and lands that many calls of
//! `place_frames` later, after everything queued before it. A placement,
//! a change of show state or of the stacking order that the table says
//! goes behind what may be queued is queued too, as a platform posts it,
//! and lands as late, after what was queued before it: for a window whose
//! `frame_delay` is 0, and whose frames land at once, on the next call of
//! `place_frames`, or on `catch_up`. A window with `absorbs_frames` takes
//! its frames without moving, and a `hung` one gets to nothing queued.
//!
//! [`FrameTable`]: crate::frames::FrameTable

use std::time::Instant;

use crate::WindowId;
use crate::frames::{Frame, FrameOutcome};

#[path = "fake_desktop_probe.rs"]
mod probe;
#[path = "fake_desktop_queued.rs"]
mod queued;

use probe::Probe;
pub(crate) use queued::Queued;
use queued::land;

use super::{FakeDesktop, FakeWindow};

impl FakeDesktop {
    /// Moves each window to its frame at `now`, the way `place_frames`
    /// does, once what was due has landed.
    pub(crate) fn take_frames(
        &mut self,
        frames: &[Frame],
        now: Instant,
        outcomes: &mut Vec<FrameOutcome>,
    ) {
        self.land_due();
        let mut probe = Probe {
            windows: &mut self.windows,
        };
        self.frames.place(frames, &mut probe, now, outcomes);
    }

    /// Whether a change made to `window` at `now` goes behind what may be
    /// queued for it, as `FrameControl::posts_behind` says.
    pub(crate) fn posts_behind(&mut self, window: WindowId, now: Instant) -> bool {
        let probe = Probe {
            windows: &mut self.windows,
        };
        self.frames.posts_behind(window, &probe, now)
    }

    /// Makes `change` to `window` at `now`: at once, unless the table
    /// says it goes behind what may be queued for the window, as a
    /// platform posts it, and notes it as a platform does.
    pub(super) fn change(&mut self, window: WindowId, change: Queued, now: Instant) {
        let behind = self.posts_behind(window, now);
        let probe = Probe {
            windows: &mut self.windows,
        };
        match (change.in_view(), behind) {
            (Some(in_view), true) => self.frames.posted_show_state(window, &probe, now, in_view),
            (Some(_), false) => self.frames.show_state_made(window, &probe),
            (None, true) => self.frames.posted(window, &probe, now),
            (None, false) => {}
        }
        let change = match change {
            Queued::Place(rect) if behind && self.stays_maximized(window) => {
                Queued::PlaceMaximized(rect)
            }
            _ => change,
        };
        match self.windows.get_mut(&window) {
            Some(w) if behind => w.queued.push((change, w.frame_delay)),
            _ => self.land_on(window, change),
        }
    }

    /// Whether a placement posted to `window` now leaves it maximized, as
    /// `FrameControl` allows for one maximized, out of view or to be once
    /// what was posted lands, whose application does not respond or that
    /// is still in view.
    fn stays_maximized(&mut self, window: WindowId) -> bool {
        let probe = Probe {
            windows: &mut self.windows,
        };
        let in_view = self.frames.in_view_once_landed(window, &probe);
        let out_of_view = |w: &FakeWindow| !in_view.unwrap_or(w.visible);
        let w = self.windows.get(&window);
        w.is_some_and(|w| w.maximized && !w.minimized && (w.hung || w.visible) && out_of_view(w))
    }

    /// Forgets the windows past the grace whose application responds, as
    /// `FrameControl::settle_responding` says.
    pub(crate) fn settle_responding(&mut self, now: Instant) {
        let probe = Probe {
            windows: &mut self.windows,
        };
        self.frames.settle_responding(&probe, now);
    }

    /// The application of `window` gets to everything queued for it.
    pub(crate) fn catch_up(&mut self, window: WindowId) {
        let queued = self
            .windows
            .get_mut(&window)
            .map(|w| std::mem::take(&mut w.queued));
        for (queued, _) in queued.unwrap_or_default() {
            self.land_on(window, queued);
        }
    }

    /// Lands what has been queued long enough, in order, and counts down
    /// the rest, for every window whose application responds.
    fn land_due(&mut self) {
        let mut landing = Vec::new();
        for (&window, w) in self.windows.iter_mut().filter(|(_, w)| !w.hung) {
            let due = w.queued.iter().take_while(|(_, left)| *left == 0).count();
            landing.extend(w.queued.drain(..due).map(|(queued, _)| (window, queued)));
            for (_, left) in &mut w.queued {
                *left = left.saturating_sub(1);
            }
        }
        for (window, queued) in landing {
            self.land_on(window, queued);
        }
    }

    /// What `queued` does to `window` and the stacking order when its
    /// application gets to it.
    fn land_on(&mut self, window: WindowId, queued: Queued) {
        match queued {
            Queued::PutBehind(other) => self.put_behind(window, other),
            _ => {
                if let Some(w) = self.windows.get_mut(&window) {
                    land(w, queued);
                }
            }
        }
    }
}
