use crate::{MonitorId, MonitorKey, Rect, WindowId};

use super::WindowIdentity;

/// Read-only view of the desktop the tiler decides against.
///
/// The tiler never changes the desktop itself: every change it wants is
/// returned as an [`Effect`](super::Effect). This trait is the only way
/// it learns about the current state, which is what lets tests drive the
/// tiler against a fake desktop.
///
/// # Contract for platform implementations
///
/// - Every method is a query. None may change a window, the cursor or
///   anything else the user can see.
/// - Answers describe the desktop right now. The tiler accounts for its
///   own effects that have not been applied yet, so a platform must not
///   try to predict them. A change posted as
///   [`posts_behind`](crate::frames::FrameControl::posts_behind) says may
///   not show until the window gets to it, so the tiler must not read it
///   back as done.
/// - A window that no longer exists answers like an invisible one:
///   `is_visible` false, `rect`, `owner`, `identity` and `facts` `None`.
///   Its `monitor_of` may be an id that is in no [`Display`] list.
pub trait Desktop {
    /// Whether the window is shown. A window that is only hidden by the
    /// system's own desktop switching still counts as shown.
    fn is_visible(&self, window: WindowId) -> bool;

    /// Whether the window is minimized.
    fn is_minimized(&self, window: WindowId) -> bool;

    /// Whether the window is maximized.
    fn is_maximized(&self, window: WindowId) -> bool;

    /// The window's visible bounds, as the user sees them.
    fn rect(&self, window: WindowId) -> Option<Rect>;

    /// The window that owns this one, for dialogs and property sheets.
    fn owner(&self, window: WindowId) -> Option<WindowId>;

    /// Who the window belongs to: what tells it apart from a later
    /// window that is given the same id.
    fn identity(&self, window: WindowId) -> Option<WindowIdentity>;

    /// Session id of the monitor the window is on.
    ///
    /// For a window that exists, one of the ids in the latest [`Display`]
    /// list: the monitor that holds the largest part of the window, the
    /// one a minimized window will be restored to, or the primary monitor
    /// for a window that is on none. A window that no longer exists may
    /// report an id that is in no list.
    fn monitor_of(&self, window: WindowId) -> MonitorId;

    /// Pointer position in desktop coordinates, if it can be read.
    fn cursor(&self) -> Option<(i32, i32)>;

    /// What the tiler needs to judge whether to manage the window.
    ///
    /// Returns `None` when the platform rules the window out on its own:
    /// it is not an application window (tool windows, owned dialogs,
    /// shell surfaces), the system is keeping it out of view, or the
    /// platform cannot position it. Visibility is not part of this
    /// check, so a window that exists but has not been shown yet can
    /// still be judged.
    fn candidate(&self, window: WindowId) -> Option<WindowFacts>;

    /// What the rules match against, for a window the tiler hid itself.
    ///
    /// [`candidate`](Self::candidate) may rule such a window out for
    /// being out of view, which is the tiler's own doing, so the tiler
    /// asks this instead when it judges a window it hid, for example
    /// when the rules change. Describe the window the way `candidate`
    /// does once it is in view, but do not rule it out for being out of
    /// view, nor for being out of reach: the tiler only hides windows it
    /// took in, and it could position them then. `None` only when the
    /// window no longer exists or is no longer an application window.
    fn facts(&self, window: WindowId) -> Option<WindowFacts>;
}

/// Properties the window rules match against.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WindowFacts {
    /// Window class or the platform's nearest equivalent.
    pub class: String,
    /// Window title.
    pub title: String,
    /// Full path of the executable that owns the window, or empty.
    pub path: String,
    /// Whether the window has a title bar. Windows without one are only
    /// managed when a rule asks for it.
    pub has_caption: bool,
}

/// A connected monitor, as [`Enumerator`](super::Enumerator) lists it.
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
