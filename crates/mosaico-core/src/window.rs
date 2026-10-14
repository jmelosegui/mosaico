mod behind;

pub use behind::Behind;

use crate::config::HidingBehaviour;
use crate::{HidingEvents, Rect, Result, WindowId};

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
    /// - A window [`hide`](Self::hide) took out of view with
    ///   [`HidingBehaviour::Hide`] is moved and stays out of view, so
    ///   showing it afterwards shows it in place.
    /// - A minimized window stays minimized, and placing it need not
    ///   change where it goes once restored: the tiler restores a window
    ///   before it places it.
    /// - The window is not raised or focused. Whatever its drawing needs
    ///   after the move is left to [`repaint`](Self::repaint).
    /// - The move must not be reported as the user moving the window
    ///   ([`WindowEvent::Moved`](crate::WindowEvent::Moved)). Reports
    ///   that its position changed are fine.
    fn place(&self, window: WindowId, rect: Rect) -> Result<()>;

    /// Moves and resizes a window so its visible bounds are `rect`,
    /// without waiting for its application.
    ///
    /// Used for a window that exists but has not been shown yet, and for
    /// one that is already shown and has just been taken in.
    ///
    /// - Returns at once, even when the application never handles the
    ///   request. The move lands when the application next handles its
    ///   messages, or not at all.
    /// - The window is not shown, raised or focused, and nothing is
    ///   repainted beyond what the move itself causes.
    /// - The move must not be reported as the user moving the window
    ///   ([`WindowEvent::Moved`](crate::WindowEvent::Moved)). Reports
    ///   that its position changed are fine.
    /// - A maximized window may stay maximized.
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

    /// Puts `window` behind `other` in the stacking order, within its own
    /// always-on-top layer.
    ///
    /// - Neither window's always-on-top layer changes: one that is
    ///   always on top stays so, and one that is not does not become so.
    ///   `window` is ordered behind `other` within its own layer, as
    ///   [`Behind`] says: directly behind it when they share a layer, and
    ///   left where it is otherwise. That is behind `other` already when
    ///   only `other` is always on top. When only `window` is, it stays
    ///   in front of `other`, and covers it where they overlap.
    /// - Neither window is activated, moved or resized, and the window
    ///   that has focus keeps it. This is the only way to change which
    ///   window is in front without focusing one.
    /// - Fails when either window no longer exists.
    fn put_behind(&self, window: WindowId, other: WindowId) -> Result<()>;

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
    /// - Afterwards the desktop reports the window as the
    ///   [`HidingBehaviour`] docs say: still shown when cloaked, not
    ///   shown when hidden, minimized when minimized. The tiler plans
    ///   the rest of a switch on that.
    /// - The window is reported hidden or minimized only when
    ///   [`hiding_events`](Self::hiding_events) says so for `how`.
    fn hide(&self, window: WindowId, how: HidingBehaviour) -> Result<()>;

    /// Brings back a window that [`hide`](Self::hide) hid the same way,
    /// without focusing it.
    ///
    /// The window is reported restored only when
    /// [`hiding_events`](Self::hiding_events) says so for `how`.
    fn show(&self, window: WindowId, how: HidingBehaviour) -> Result<()>;

    /// Whether the platform can hide windows `how`, and what hiding and
    /// showing a window that way reports; `None` when it cannot.
    ///
    /// Asked about every way once, when tiling starts, see
    /// [`HidingSupport`](crate::HidingSupport). The tiler hides windows
    /// only in ways this answers for, unless it answers `None` for all
    /// of them, and takes the hide and restore reports it says to expect
    /// for its own. Focus reports for a window just hidden are taken for
    /// its own whatever the answer.
    ///
    /// - Answer `None` for a way the platform cannot make look the way
    ///   [`HidingBehaviour`] describes it, rather than carry it out as
    ///   another way: the tiler chooses what to use instead, and says so.
    ///   [`hide`](Self::hide) and [`show`](Self::show) return
    ///   [`Error::Unsupported`](crate::Error::Unsupported) for such a way.
    /// - The answer must not change while the process runs.
    /// - Have at least one way, or workspace switches leave the windows
    ///   of the workspace left on screen.
    /// - The events must be the ones the platform really reports. One it
    ///   reports without saying so is taken for the user or the
    ///   application: a hide can make the tiler stop managing the window,
    ///   a restore makes it focus the window. One it says so about but
    ///   never reports makes the tiler ignore the application's own, until
    ///   the window is shown again for a hide, or briefly after that for
    ///   a restore.
    /// - Showing a window may also report it shown, as
    ///   [`WindowEvent::Created`](crate::WindowEvent::Created). There is
    ///   nothing to answer about that: the tiler ignores it for a window
    ///   it already manages.
    fn hiding_events(&self, how: HidingBehaviour) -> Option<HidingEvents>;

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
