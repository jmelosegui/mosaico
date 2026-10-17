//! The desktop as the tiler sees it while changes of show state may be on
//! their way.
//!
//! A hide, a minimize or a show posted behind what is on its way to a
//! window happens only once the window gets to it, and meanwhile the
//! desktop still has the window as it was. The tiler made the change, so
//! its view has the window as the change leaves it: one whose minimize is
//! queued is minimized already, so a display change restores it before it
//! places it, and one whose show is queued is shown, so the layout does not
//! take it for one the user hid. A change is kept only when it is posted
//! and changes something: the desktop does not have the window as it
//! leaves it yet, or, for a show, a hide of it is still to be reported, as
//! it lands later. It is forgotten once neither holds, once the window
//! reports it, once nothing sent to the window may still be on its way, as
//! for a new window given its id, with the next change of the window's
//! show state, and with the window's end.

use std::time::Instant;

use crate::WindowId;
use crate::config::HidingBehaviour;

use super::super::{Desktop, Effect, Expectations, Platform};
use super::Applier;
use super::view::PostedView;

/// How a change of a window's show state leaves it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ShowState {
    /// Shown, and not minimized.
    Shown,
    /// Not shown.
    Hidden,
    /// Minimized.
    Minimized,
}

impl ShowState {
    /// Whether `desktop` has `window` as this leaves it.
    fn holds(self, desktop: &dyn Desktop, window: WindowId) -> bool {
        match self {
            Self::Shown => desktop.is_visible(window) && !desktop.is_minimized(window),
            Self::Hidden => !desktop.is_visible(window),
            Self::Minimized => desktop.is_minimized(window),
        }
    }
}

impl Applier {
    /// Notes a change of `window`'s show state just made, that leaves it
    /// `to`, through the window's own queue or not, as cloaking does not. It
    /// is kept while it changes something and was posted, as `platform` says
    /// right after it: a posted change renews the grace, so the platform
    /// says so then for one that was posted, or for a window whose
    /// application stopped responding since, which waits the same way.
    pub(super) fn show_state(
        &mut self,
        window: WindowId,
        (to, queued): (ShowState, bool),
        platform: &dyn Platform,
    ) {
        let posted = queued && platform.control().posts_behind(window);
        let late_hide = to == ShowState::Shown && self.expectations.hide_unreported(window);
        if posted && (!to.holds(platform.desktop(), window) || late_hide) {
            self.posted.states.insert(window, to);
        } else {
            self.posted.states.remove(&window);
        }
        if to != ShowState::Shown {
            self.forget_deferred_border(window);
        }
    }

    /// The tiler's view of the desktop of `platform` at `now`, where each
    /// window has the show state the change posted to it leaves it in, once
    /// the changes that landed, or that nothing may still be on its way to,
    /// are forgotten. A show is kept while a hide of its window is still to
    /// be reported, even where the desktop has the window shown: the hide
    /// is still to land.
    pub(in crate::tiler) fn view<'a>(
        &'a mut self,
        platform: &'a dyn Platform,
        now: Instant,
    ) -> PostedView<'a> {
        let expectations = &self.expectations;
        let on_its_way = |&window: &WindowId, to: &mut ShowState| {
            on_its_way(expectations, platform, window, *to, now)
        };
        self.posted.states.retain(on_its_way);
        PostedView::new(platform.desktop(), platform.control(), &self.posted.states)
    }

    /// Reads the desktop of `platform` at `now` as [`view`](Self::view)
    /// has it, through `read`, without forgetting the changes no longer on
    /// their way: for the bars, which read it between steps.
    pub(in crate::tiler) fn read_view<R>(
        &self,
        platform: &dyn Platform,
        now: Instant,
        read: impl FnOnce(&dyn Desktop) -> R,
    ) -> R {
        let states = (self.posted.states.iter())
            .filter(|&(&window, &to)| on_its_way(&self.expectations, platform, window, to, now))
            .map(|(&window, &to)| (window, to))
            .collect();
        read(&PostedView::new(
            platform.desktop(),
            platform.control(),
            &states,
        ))
    }
}

/// Whether the change that leaves `window` as `to` is still on its way at
/// `now`, as `expectations` and `platform` have it: posted, and either not
/// holding yet or, for a show, with a hide of the window still to be
/// reported.
fn on_its_way(
    expectations: &Expectations,
    platform: &dyn Platform,
    window: WindowId,
    to: ShowState,
    now: Instant,
) -> bool {
    let late_hide = to == ShowState::Shown && expectations.hide_expected(window, now);
    (late_hide || !to.holds(platform.desktop(), window)) && platform.control().posts_behind(window)
}

/// How `effect`, a hide, a minimize, a show or a reveal, leaves its window,
/// and whether it goes through the window's own queue, as cloaking does
/// not.
pub(super) fn left_by(effect: &Effect) -> (ShowState, bool) {
    match *effect {
        Effect::Hide {
            how: HidingBehaviour::Minimize,
            ..
        }
        | Effect::Minimize { .. } => (ShowState::Minimized, true),
        Effect::Hide { how, .. } => (ShowState::Hidden, how != HidingBehaviour::Cloak),
        Effect::Show { how, .. } => (ShowState::Shown, how != HidingBehaviour::Cloak),
        _ => (ShowState::Shown, true),
    }
}
