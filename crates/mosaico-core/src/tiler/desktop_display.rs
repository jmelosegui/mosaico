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
    /// The whole monitor in desktop pixels. Taskbars, docks and panels at
    /// its edges are inside it, and so is [`work_area`](Self::work_area).
    pub bounds: Rect,
    /// Usable area, excluding taskbars, docks and panels.
    pub work_area: Rect,
    /// Whether this is the primary monitor.
    pub primary: bool,
    /// What the system calls the display, as shown to the user, such as
    /// "DELL U2720Q". Empty when the platform cannot tell.
    pub name: String,
    /// The name the system gives the display's output, which other
    /// programs use to find it. Empty when unknown.
    pub device: String,
    /// The display's scale factor: 1.0 at 100%, 1.5 at 150%. 1.0 when
    /// unknown.
    pub scale: f64,
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
