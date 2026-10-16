//! Listing the windows and monitors on the desktop.

use crate::{MonitorId, Rect, Result, WindowId};

use super::{Display, WindowFacts};

/// Lists what is on the desktop: application windows and monitors.
///
/// The tiler never lists anything itself. The code that drives it takes
/// these lists at startup and after display changes and hands them in
/// ([`Tiler::new`](super::Tiler::new),
/// [`Input::Displays`](super::Input::Displays),
/// [`Input::WorkAreas`](super::Input::WorkAreas)). The bar and the
/// diagnostic commands use them too. Questions about one window, asked
/// while the tiler decides, go to [`Desktop`](super::Desktop).
///
/// # Contract for platform implementations
///
/// - Every method is a query. None may change a window, a monitor or
///   anything else the user can see.
/// - Lists describe the desktop at the time of the call. A window or
///   monitor that goes away while it is being listed is left out.
/// - Implement [`connected_monitors`](Self::connected_monitors) and keep
///   the provided [`monitors`](Self::monitors): the order is decided
///   here, so every platform numbers monitors the same way.
pub trait Enumerator {
    /// The top-level application windows that are shown and not
    /// minimized, front to back.
    ///
    /// # Contract for platform implementations
    ///
    /// - Lists every top-level window that is shown and not minimized,
    ///   except tool palettes and floating toolbars, windows owned by
    ///   another window (dialogs, property sheets) and modal dialogs.
    /// - Windows without a title bar are listed. The window rules decide
    ///   whether they are tiled.
    /// - Makes no other judgement. Windows the system keeps out of view,
    ///   such as those on another virtual desktop, and windows the
    ///   platform cannot move are listed;
    ///   [`Desktop::candidate`](super::Desktop::candidate) rules them out.
    /// - Front to back when the platform knows the stacking order,
    ///   otherwise in an order that stays the same between calls. The
    ///   tiler lays windows out in this order at startup.
    /// - Fails only when the list cannot be read at all. A platform that
    ///   cannot list windows returns
    ///   [`Error::Unsupported`](crate::Error::Unsupported).
    fn windows(&self) -> Result<Vec<WindowId>>;

    /// Every connected monitor, in whatever order the platform finds
    /// them. Callers use [`monitors`](Self::monitors), which puts them in
    /// order.
    ///
    /// # Contract for platform implementations
    ///
    /// - Lists every monitor that is part of the desktop. Monitors that
    ///   mirror each other count as one.
    /// - While the monitors stay the same, every call lists them in the
    ///   same order. Monitor numbers depend on it for monitors that share
    ///   a left edge, and so does the tiler when it matches fresh work
    ///   areas by position.
    /// - At most one is primary. There is none only when the primary
    ///   monitor is left out because it could not be read. A platform
    ///   without the notion of a primary monitor marks the one that holds
    ///   the desktop origin.
    /// - Ids follow the [`MonitorId`] contract and keys the
    ///   [`MonitorKey`](crate::MonitorKey) contract.
    /// - Work areas leave out taskbars, docks and panels, and use the
    ///   same desktop coordinates as window bounds.
    /// - A monitor whose details cannot be read is left out. Fails only
    ///   when the list cannot be read at all. A platform that cannot list
    ///   monitors returns [`Error::Unsupported`](crate::Error::Unsupported).
    fn connected_monitors(&self) -> Result<Vec<Display>>;

    /// What the platform can tell about a window, for `mosaico debug
    /// list` and the window titles `mosaico debug events` prints.
    ///
    /// # Contract for platform implementations
    ///
    /// - Never fails. A window that no longer exists, or a detail that
    ///   cannot be read, comes back empty: blank facts, no rect, neither
    ///   minimized nor maximized, no frame.
    /// - The facts are not judged. Unlike
    ///   [`Desktop::candidate`](super::Desktop::candidate), they are
    ///   reported for windows the platform would rule out.
    /// - `monitor` follows the same rule as
    ///   [`Desktop::monitor_of`](super::Desktop::monitor_of), so for a
    ///   window that no longer exists it may be an id that is in no list.
    fn details(&self, window: WindowId) -> WindowDetails;

    /// Every connected monitor, left to right.
    ///
    /// Sorted by the left edge of the work area. Monitors that share a
    /// left edge stay in the order the platform found them in. This order
    /// is what a monitor number means everywhere: the bar's `monitors`
    /// setting, `mosaico debug list`, the tiler's monitor indices and the
    /// monitor that has focus at startup.
    fn monitors(&self) -> Result<Vec<Display>> {
        let mut monitors = self.connected_monitors()?;
        // A stable sort, which is what keeps monitors that share a left
        // edge in the platform's order.
        monitors.sort_by_key(|m| m.work_area.x);
        Ok(monitors)
    }
}

/// A window as `mosaico debug list` shows it, and where `mosaico debug
/// events` reads its titles from.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowDetails {
    /// What the window rules match against.
    pub facts: WindowFacts,
    /// The window's visible bounds, if they can be read.
    pub rect: Option<Rect>,
    /// Whether the window is minimized.
    pub minimized: bool,
    /// Whether the window is maximized.
    pub maximized: bool,
    /// Session id of the monitor the window is on.
    pub monitor: MonitorId,
    /// How far the window's outer bounds reach past its visible bounds,
    /// such as invisible resize borders or a shadow. All zero when they
    /// do not, `None` when it cannot be read.
    pub frame: Option<Insets>,
}

/// Widths of a margin along each edge of a rectangle, in pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Insets {
    /// Along the left edge.
    pub left: i32,
    /// Along the top edge.
    pub top: i32,
    /// Along the right edge.
    pub right: i32,
    /// Along the bottom edge.
    pub bottom: i32,
}

#[cfg(test)]
#[path = "tests/enumerator_tests.rs"]
mod tests;
