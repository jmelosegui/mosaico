//! An in-memory desktop for the tiler tests.

use std::collections::{HashMap, HashSet};

use crate::frames::FrameTable;
use crate::{MonitorId, Rect, WindowId};

use super::super::{Desktop, WindowFacts, WindowIdentity};

#[path = "fake_desktop_apply.rs"]
mod apply;
#[path = "fake_desktop_frames.rs"]
mod frames;
#[path = "fake_desktop_window.rs"]
mod window;

pub(crate) use frames::Queued;
pub(crate) use window::FakeWindow;

/// An in-memory desktop.
#[derive(Debug, Default)]
pub(crate) struct FakeDesktop {
    pub(crate) windows: HashMap<WindowId, FakeWindow>,
    pub(crate) cursor: Option<(i32, i32)>,
    pub(crate) focused: Option<WindowId>,
    /// Border overlays, as the platform keeps them: one per window,
    /// created when first drawn and destroyed by `DropBorder`. The value
    /// is whether the overlay is showing.
    pub(crate) borders: HashMap<WindowId, bool>,
    /// Where the drop preview shows, while it does.
    pub(crate) preview: Option<Rect>,
    /// The stacking order, front first. A window comes to the front when
    /// it opens and when it gets focus.
    pub(crate) stack: Vec<WindowId>,
    /// What the platform keeps about the frames posted to the windows.
    pub(crate) frames: FrameTable<()>,
    /// Whether the user turned animations off in the system's settings.
    pub(crate) animations_off: bool,
    /// The windows whose changes it says are posted, for the tiler on its
    /// own: through the manager, the window control says.
    pub(crate) posting: HashSet<WindowId>,
}

impl Desktop for FakeDesktop {
    fn is_visible(&self, window: WindowId) -> bool {
        self.windows.get(&window).is_some_and(|w| w.visible)
    }

    fn is_minimized(&self, window: WindowId) -> bool {
        self.windows.get(&window).is_some_and(|w| w.minimized)
    }

    fn is_maximized(&self, window: WindowId) -> bool {
        self.windows.get(&window).is_some_and(|w| w.maximized)
    }

    fn rect(&self, window: WindowId) -> Option<Rect> {
        self.windows.get(&window).map(|w| w.rect)
    }

    fn owner(&self, window: WindowId) -> Option<WindowId> {
        self.windows.get(&window).and_then(|w| w.owner)
    }

    fn identity(&self, window: WindowId) -> Option<WindowIdentity> {
        self.windows.get(&window).map(|w| WindowIdentity {
            process: w.process,
            class: w
                .facts
                .as_ref()
                .map(|f| f.class.clone())
                .unwrap_or_default(),
        })
    }

    fn monitor_of(&self, window: WindowId) -> MonitorId {
        MonitorId::from_raw(self.windows.get(&window).map_or(0, |w| w.monitor as u64))
    }

    fn cursor(&self) -> Option<(i32, i32)> {
        self.cursor
    }

    /// The windows shown, not minimized, not cloaked and not the shell's,
    /// front to back, as a platform lists them: in the stacking order, and
    /// after it, in the order of their ids, the windows it does not hold.
    fn windows_on_screen(&self) -> Vec<WindowId> {
        let mut rest: Vec<WindowId> = (self.windows.keys())
            .filter(|window| !self.stack.contains(window))
            .copied()
            .collect();
        rest.sort_by_key(|window| window.raw());
        let on_screen = |window: &WindowId| {
            (self.windows.get(window))
                .is_some_and(|w| w.visible && !w.minimized && !w.cloaked && !w.shell)
        };
        self.stack
            .iter()
            .chain(&rest)
            .copied()
            .filter(on_screen)
            .collect()
    }

    fn candidate(&self, window: WindowId) -> Option<WindowFacts> {
        self.windows
            .get(&window)
            .filter(|w| !w.cloaked)
            .and_then(|w| w.facts.clone())
    }

    fn facts(&self, window: WindowId) -> Option<WindowFacts> {
        self.windows.get(&window).and_then(|w| w.facts.clone())
    }

    fn animations_allowed(&self) -> bool {
        !self.animations_off
    }

    fn changes_posted(&self, window: WindowId) -> bool {
        self.posting.contains(&window)
    }
}
