//! The desktop of the fake [`World`] as the tiler reads it.
//!
//! No command reads windows this way, so the World has none: every
//! window answers as one that no longer exists. It only says whether the
//! system lets windows be animated, which `doctor` reports.

use mosaico_core::tiler::{Desktop, WindowFacts, WindowIdentity};
use mosaico_core::{MonitorId, Rect, WindowId};

use super::World;

impl Desktop for World {
    fn is_visible(&self, _: WindowId) -> bool {
        false
    }

    fn is_minimized(&self, _: WindowId) -> bool {
        false
    }

    fn is_maximized(&self, _: WindowId) -> bool {
        false
    }

    fn rect(&self, _: WindowId) -> Option<Rect> {
        None
    }

    fn owner(&self, _: WindowId) -> Option<WindowId> {
        None
    }

    fn identity(&self, _: WindowId) -> Option<WindowIdentity> {
        None
    }

    fn monitor_of(&self, _: WindowId) -> MonitorId {
        MonitorId::from_raw(0)
    }

    fn cursor(&self) -> Option<(i32, i32)> {
        None
    }

    fn windows_on_screen(&self) -> Vec<WindowId> {
        Vec::new()
    }

    fn candidate(&self, _: WindowId) -> Option<WindowFacts> {
        None
    }

    fn facts(&self, _: WindowId) -> Option<WindowFacts> {
        None
    }

    fn animations_allowed(&self) -> bool {
        self.animations_allowed.get()
    }
}
