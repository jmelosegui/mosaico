//! An in-memory desktop for the tiler tests.

use std::collections::HashMap;

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
    /// The stacking order, front first. A window comes to the front when
    /// it opens and when it gets focus.
    pub(crate) stack: Vec<WindowId>,
    /// What the platform keeps about the frames posted to the windows.
    pub(crate) frames: FrameTable<()>,
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

    fn candidate(&self, window: WindowId) -> Option<WindowFacts> {
        self.windows
            .get(&window)
            .filter(|w| !w.cloaked)
            .and_then(|w| w.facts.clone())
    }

    fn facts(&self, window: WindowId) -> Option<WindowFacts> {
        self.windows.get(&window).and_then(|w| w.facts.clone())
    }
}
