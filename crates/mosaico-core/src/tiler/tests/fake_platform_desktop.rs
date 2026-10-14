//! What a [`FakePlatform`] answers about the desktop: the queries the
//! tiler asks while it decides, and the lists of windows and monitors.

use crate::{Error, MonitorId, Rect, Result, WindowId};

use super::super::super::{
    Desktop, Display, Enumerator, WindowDetails, WindowFacts, WindowIdentity,
};
use super::FakePlatform;

impl Desktop for FakePlatform {
    fn is_visible(&self, window: WindowId) -> bool {
        self.shared.desktop.borrow().is_visible(window)
    }

    fn is_minimized(&self, window: WindowId) -> bool {
        self.shared.desktop.borrow().is_minimized(window)
    }

    fn is_maximized(&self, window: WindowId) -> bool {
        self.shared.desktop.borrow().is_maximized(window)
    }

    fn rect(&self, window: WindowId) -> Option<Rect> {
        self.shared.desktop.borrow().rect(window)
    }

    fn owner(&self, window: WindowId) -> Option<WindowId> {
        self.shared.desktop.borrow().owner(window)
    }

    fn identity(&self, window: WindowId) -> Option<WindowIdentity> {
        self.shared.desktop.borrow().identity(window)
    }

    fn monitor_of(&self, window: WindowId) -> MonitorId {
        self.shared.desktop.borrow().monitor_of(window)
    }

    fn cursor(&self) -> Option<(i32, i32)> {
        self.shared.desktop.borrow().cursor()
    }

    fn candidate(&self, window: WindowId) -> Option<WindowFacts> {
        self.shared.desktop.borrow().candidate(window)
    }

    fn facts(&self, window: WindowId) -> Option<WindowFacts> {
        self.shared.desktop.borrow().facts(window)
    }
}

impl Enumerator for FakePlatform {
    /// The windows that are shown and not minimized, in id order.
    fn windows(&self) -> Result<Vec<WindowId>> {
        let desktop = self.shared.desktop.borrow();
        let mut windows: Vec<WindowId> = desktop
            .windows
            .iter()
            .filter(|(_, w)| w.visible && !w.minimized)
            .map(|(&id, _)| id)
            .collect();
        windows.sort();
        Ok(windows)
    }

    fn connected_monitors(&self) -> Result<Vec<Display>> {
        if self.shared.monitors_fail.get() {
            return Err(Error::platform(
                "list monitors",
                "the fake was told to fail",
            ));
        }
        Ok(self.shared.displays.borrow().clone())
    }

    fn details(&self, window: WindowId) -> WindowDetails {
        let desktop = self.shared.desktop.borrow();
        let found = desktop.windows.get(&window);
        WindowDetails {
            facts: found.and_then(|w| w.facts.clone()).unwrap_or_default(),
            rect: found.map(|w| w.rect),
            minimized: found.is_some_and(|w| w.minimized),
            maximized: found.is_some_and(|w| w.maximized),
            monitor: desktop.monitor_of(window),
            frame: None,
        }
    }
}
