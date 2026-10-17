//! A desktop with no windows, which is all the tiler needs to reserve
//! space for the bars.

use crate::tiler::{Desktop, WindowFacts, WindowIdentity};
use crate::{MonitorId, Rect, WindowId};

/// A desktop with no windows.
pub(super) struct NoWindows;

impl Desktop for NoWindows {
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
}
