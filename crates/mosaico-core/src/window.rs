use crate::config::HidingBehaviour;
use crate::{Rect, Result, WindowId};

/// Platform-agnostic window trait.
///
/// Each platform crate (e.g. `mosaico-windows`) provides its own implementation.
///
/// # Contract for platform implementations
///
/// - The methods that return a [`Result`] fail with
///   [`Error::Platform`](crate::Error::Platform) when the OS refuses the
///   call, for example because the window has closed since it was found.
pub trait Window {
    /// Returns the window title.
    fn title(&self) -> Result<String>;

    /// Returns the window class name.
    fn class(&self) -> Result<String>;

    /// Returns the window bounding rectangle.
    fn rect(&self) -> Result<Rect>;

    /// Moves and resizes the window to the given rectangle.
    ///
    /// Takes `&self` because the mutation happens on the OS side via the
    /// window handle — the Rust struct itself doesn't change.
    fn set_rect(&self, rect: &Rect) -> Result<()>;

    /// Forces the window to repaint, repairing its rendering if needed.
    ///
    /// Some applications (e.g. Chromium-based browsers) need an explicit
    /// redraw after being repositioned programmatically, and can draw at
    /// a stale offset after a display change. The tiler calls this after
    /// positioning a window and for every visible window after a display
    /// change or a retile, so platforms put such repairs here. A platform
    /// with nothing to repair may leave it a plain repaint.
    fn invalidate(&self);

    /// Returns whether the window is currently visible.
    fn is_visible(&self) -> bool;
}

/// Changes the platform makes to windows, and to the pointer.
///
/// The tiler never changes the desktop itself: it returns the changes
/// it wants as effects, and the ones that move, show, hide, focus or
/// close a window reach the platform through this trait, by way of
/// [`apply_window_effect`](crate::tiler::apply_window_effect). `debug
/// move` places windows through it too. Reading the desktop is the job
/// of [`Desktop`](crate::tiler::Desktop).
///
/// # Contract for platform implementations
///
/// - Rects are a window's visible bounds, in the same coordinates and
///   units as [`Desktop::rect`](crate::tiler::Desktop::rect) and the
///   monitor work areas: no invisible resize borders or shadows.
/// - Report a failure the platform can detect, such as a window that
///   no longer exists or a request the system refused, as an error.
///   Never panic: callers log the error and carry on. Return
///   [`Error::Unsupported`](crate::Error::Unsupported) for something the
///   platform cannot do at all.
/// - Only [`focus`](Self::focus) and [`reveal`](Self::reveal) may give
///   focus to the window they act on. Hiding, minimizing or closing a
///   window may let the system activate another one, as it would for
///   the user.
/// - Only [`pre_place`](Self::pre_place) and [`close`](Self::close)
///   promise not to wait for the application. The others may wait for
///   it to handle the change.
pub trait WindowControl {
    /// Moves and resizes a window so its visible bounds are `rect`.
    ///
    /// - A maximized window is no longer maximized afterwards.
    /// - The window is not raised or focused. Whatever its drawing needs
    ///   after the move is left to [`repaint`](Self::repaint).
    /// - The move must not be reported as the user moving the window
    ///   ([`WindowEvent::Moved`](crate::WindowEvent::Moved)). Reports
    ///   that its position changed are fine.
    fn place(&self, window: WindowId, rect: Rect) -> Result<()>;

    /// Moves a window that exists but has not been shown yet.
    ///
    /// - Returns at once, even when the application never handles the
    ///   request. The move may land later, or not at all.
    /// - The window is not shown, raised or focused.
    fn pre_place(&self, window: WindowId, rect: Rect) -> Result<()>;

    /// Repaints a window without moving it, repairing its rendering if
    /// needed.
    ///
    /// Some applications need an explicit redraw after being positioned
    /// programmatically, and can draw at a stale offset after a display
    /// change. The tiler repaints a window after placing it and every
    /// visible window after a display change or a retile, so platforms
    /// put such repairs here. A platform with nothing to repair may make
    /// it a plain repaint.
    fn repaint(&self, window: WindowId) -> Result<()>;

    /// Gives a window keyboard focus and brings it to the front.
    ///
    /// The system may refuse, to stop applications from stealing focus.
    /// Does not move the pointer.
    fn focus(&self, window: WindowId) -> Result<()>;

    /// Asks a window to close, the way the user closing it would.
    ///
    /// The application may save its work, ask the user, or refuse. The
    /// platform never destroys the window or ends its process itself.
    fn close(&self, window: WindowId) -> Result<()>;

    /// Minimizes a window.
    ///
    /// The platform must report it back as
    /// [`WindowEvent::Minimized`](crate::WindowEvent::Minimized): that
    /// report is what takes the window out of the layout.
    fn minimize(&self, window: WindowId) -> Result<()>;

    /// Takes a window out of view for a workspace switch, the way `how`
    /// describes. The window stays open and where it is.
    ///
    /// Hiding with [`HidingBehaviour::Cloak`] must not be reported as the
    /// window hiding or minimizing: the tiler would take that for the
    /// application doing it and stop managing the window. The reports
    /// the other ways cause are expected and ignored.
    fn hide(&self, window: WindowId, how: HidingBehaviour) -> Result<()>;

    /// Brings back a window that [`hide`](Self::hide) hid the same way,
    /// without focusing it.
    fn show(&self, window: WindowId, how: HidingBehaviour) -> Result<()>;

    /// Makes a window visible however it was hidden, by this daemon or
    /// an earlier one. Used when the tiler stops, so that nothing is left
    /// out of view.
    ///
    /// May focus the window. A minimized window may stay minimized.
    fn reveal(&self, window: WindowId) -> Result<()>;

    /// Puts the pointer at `(x, y)` in desktop coordinates.
    ///
    /// A platform that does not let applications move the pointer
    /// returns [`Error::Unsupported`](crate::Error::Unsupported).
    fn move_cursor(&self, x: i32, y: i32) -> Result<()>;
}
