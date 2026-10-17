//! A connected monitor, as the enumerator lists it.

use crate::{MonitorId, MonitorKey, Rect};

/// A connected monitor, as [`Enumerator`](crate::tiler::Enumerator) lists
/// it.
#[derive(Debug, Clone, PartialEq)]
pub struct Display {
    /// Session id. Platforms may hand out new ids on every display
    /// change, so this only identifies a monitor until the next one.
    pub id: MonitorId,
    /// Identity that survives reconnects. See [`MonitorKey`].
    pub key: MonitorKey,
    /// Usable area, excluding taskbars, docks and panels.
    pub work_area: Rect,
    /// Whether this is the primary monitor.
    pub primary: bool,
}

impl crate::Monitor for Display {
    fn key(&self) -> &MonitorKey {
        &self.key
    }

    fn work_area(&self) -> Rect {
        self.work_area
    }

    fn is_primary(&self) -> bool {
        self.primary
    }
}
