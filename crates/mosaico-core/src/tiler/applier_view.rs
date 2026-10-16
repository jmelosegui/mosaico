//! The tiler's view of the desktop while changes of show state may be on
//! their way to its windows, as `applier_shows.rs` keeps them.

use std::collections::HashMap;

use crate::{MonitorId, Rect, WindowControl, WindowId};

use super::super::{Desktop, WindowFacts, WindowIdentity};
use super::shows::ShowState;

/// The desktop, where a window with a change of show state on its way is
/// as that change leaves it, and whose changes are posted as the window
/// control says. Everything else is the desktop's own answer.
pub(in crate::tiler) struct PostedView<'a> {
    desktop: &'a dyn Desktop,
    control: &'a dyn WindowControl,
    states: &'a HashMap<WindowId, ShowState>,
}

impl<'a> PostedView<'a> {
    /// `desktop`, changed through `control`, with the show states of
    /// `states`.
    pub(super) fn new(
        desktop: &'a dyn Desktop,
        control: &'a dyn WindowControl,
        states: &'a HashMap<WindowId, ShowState>,
    ) -> Self {
        Self {
            desktop,
            control,
            states,
        }
    }
}

impl Desktop for PostedView<'_> {
    fn changes_posted(&self, window: WindowId) -> bool {
        self.control.posts_behind(window)
    }

    fn is_visible(&self, window: WindowId) -> bool {
        match self.states.get(&window) {
            Some(ShowState::Shown) => true,
            Some(ShowState::Hidden) => false,
            _ => self.desktop.is_visible(window),
        }
    }

    fn is_minimized(&self, window: WindowId) -> bool {
        match self.states.get(&window) {
            Some(ShowState::Shown) => false,
            Some(ShowState::Minimized) => true,
            _ => self.desktop.is_minimized(window),
        }
    }

    fn is_maximized(&self, window: WindowId) -> bool {
        self.desktop.is_maximized(window)
    }

    fn rect(&self, window: WindowId) -> Option<Rect> {
        self.desktop.rect(window)
    }

    fn owner(&self, window: WindowId) -> Option<WindowId> {
        self.desktop.owner(window)
    }

    fn identity(&self, window: WindowId) -> Option<WindowIdentity> {
        self.desktop.identity(window)
    }

    fn monitor_of(&self, window: WindowId) -> MonitorId {
        self.desktop.monitor_of(window)
    }

    fn cursor(&self) -> Option<(i32, i32)> {
        self.desktop.cursor()
    }

    fn candidate(&self, window: WindowId) -> Option<WindowFacts> {
        self.desktop.candidate(window)
    }

    fn facts(&self, window: WindowId) -> Option<WindowFacts> {
        self.desktop.facts(window)
    }

    fn animations_allowed(&self) -> bool {
        self.desktop.animations_allowed()
    }
}
