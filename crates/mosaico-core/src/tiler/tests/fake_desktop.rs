//! An in-memory desktop for the tiler tests.

use std::collections::HashMap;

use crate::{MonitorId, Rect, WindowId};

use super::super::{Desktop, WindowFacts, WindowIdentity};
use super::MON_W;

#[path = "fake_desktop_apply.rs"]
mod apply;

/// A window on the fake desktop.
#[derive(Debug, Clone)]
pub(crate) struct FakeWindow {
    pub(crate) visible: bool,
    pub(crate) minimized: bool,
    pub(crate) maximized: bool,
    pub(crate) cloaked: bool,
    pub(crate) rect: Rect,
    pub(crate) owner: Option<WindowId>,
    pub(crate) monitor: usize,
    pub(crate) facts: Option<WindowFacts>,
    pub(crate) process: u32,
}

impl FakeWindow {
    /// A visible, captioned application window on monitor `monitor`.
    pub(crate) fn app(monitor: usize) -> Self {
        Self {
            visible: true,
            minimized: false,
            maximized: false,
            cloaked: false,
            rect: Rect::new(monitor as i32 * MON_W + 100, 100, 640, 480),
            owner: None,
            monitor,
            facts: Some(WindowFacts {
                class: "App".into(),
                title: "app".into(),
                path: "/apps/app".into(),
                has_caption: true,
            }),
            process: 1000,
        }
    }
}

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
