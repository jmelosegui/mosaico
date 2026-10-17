//! The contract for drawing borders around windows, the preview of where
//! a dragged window lands, and choosing how their corners look.

use crate::config::CornerStyle;
use crate::tiler::{BorderStyle, PreviewStyle};
use crate::{Rect, Result, WindowId};

/// What the platform draws on and around managed windows: a border
/// around each one, the style of its corners, and the preview of where a
/// window the user drags lands.
///
/// The tiler decides which windows have a border, how it looks and which
/// corners windows get. It returns those decisions as effects, and the
/// ones that draw borders or change corners reach the platform through
/// this trait, by way of
/// [`apply_decoration_effect`](crate::tiler::apply_decoration_effect).
/// Changes to the windows themselves go through
/// [`WindowControl`](crate::WindowControl).
///
/// # Contract for platform implementations
///
/// - A border surrounds the visible bounds of its window, the `rect` of
///   its [`BorderStyle`], which follows the rule of
///   [`WindowControl`](crate::WindowControl): no invisible resize
///   borders or shadows. The stroke lies outside the rect, `width`
///   pixels wide in the same pixels as the rect, and its outer corners
///   are rounded by `radius`. A width of 0 draws nothing.
/// - A border is only ever seen. It never takes focus or input, which
///   goes to whatever is under it, never shows as a window of its own
///   in the taskbar or the window switcher, and drawing, moving or
///   hiding it never causes a [`WindowEvent`](crate::WindowEvent).
/// - Each time it is drawn, a border goes directly behind its window,
///   so it never covers that window or the windows in front of it.
/// - A window has at most one border. Drawing it again moves, resizes
///   and recolors the one it has.
/// - Corner styles change only how a window's own corners are drawn.
///   The window is not moved, resized, raised or focused.
/// - Report a failure the platform can detect, such as a window that no
///   longer exists, as an error, unless the method says otherwise. Never
///   panic: callers log the error and carry on. A platform that cannot
///   draw borders, or cannot change the corners of windows, returns
///   [`Error::Unsupported`](crate::Error::Unsupported) from the methods
///   that would.
pub trait Decorations {
    /// Draws the border around `window` the way `style` says, creating
    /// it the first time.
    ///
    /// Fails only when the border cannot be created. A window that
    /// closed since the tiler decided to draw its border is not an
    /// error: the tiler drops the border once it learns the window is
    /// gone.
    fn show_border(&mut self, window: WindowId, style: &BorderStyle) -> Result<()>;

    /// Hides the border around `window`, keeping it to draw again. Does
    /// nothing for a window that has no border.
    fn hide_border(&mut self, window: WindowId) -> Result<()>;

    /// Moves the border around `window`, as it was last drawn, so that it
    /// surrounds `rect`, the window's visible bounds.
    ///
    /// Drawing a border can take tens of milliseconds for a large window,
    /// and moving one costs about what moving a window does. So the tiler
    /// moves the border of a window that keeps its size on its way through
    /// a transition along with it, and draws it with
    /// [`show_border`](Self::show_border) once the window is in place.
    ///
    /// - Only where the border is changes. It keeps the picture and the
    ///   size it was drawn with, its place in the stacking order, and
    ///   stays hidden if it is hidden.
    /// - Does nothing for a window that has no border.
    /// - A platform that cannot move a border without drawing it again
    ///   returns [`Error::Unsupported`](crate::Error::Unsupported), which
    ///   is the default. Such a border stays where it is until it is drawn
    ///   again.
    fn move_border(&mut self, window: WindowId, rect: Rect) -> Result<()> {
        let _ = (window, rect);
        Err(crate::Error::Unsupported(
            "moving a border without drawing it",
        ))
    }

    /// Removes the border around `window` for good, releasing what it
    /// holds. Does nothing for a window that has no border. The tiler
    /// drops the border of a window that leaves the screen.
    fn drop_border(&mut self, window: WindowId) -> Result<()>;

    /// Forgets how every border was drawn, so that the next
    /// [`show_border`](Self::show_border) for each window draws it from
    /// scratch, even when it would look as it does now, and so does the
    /// next [`show_preview`](Self::show_preview). What is on screen stays
    /// as it is until then.
    ///
    /// A platform may keep what it drew of a border and, when the border
    /// is drawn again the same way, only move it. What it kept can be
    /// lost outside mosaico, as when the display changes, so the tiler
    /// asks for this on a retile the user asks for and on a display change
    /// that changes anything. A platform that draws every border from
    /// scratch has nothing to forget.
    fn forget_drawn(&mut self);

    /// Draws the corners of `window` the way `corners` says, see
    /// [`CornerStyle`].
    fn set_corners(&mut self, window: WindowId, corners: CornerStyle) -> Result<()>;

    /// Gives `window` back the corners the system draws by default.
    fn reset_corners(&mut self, window: WindowId) -> Result<()>;

    /// Shows the drop preview around `rect`, the visible bounds `window`,
    /// which the user drags, would get if they dropped it now, or moves
    /// it there.
    ///
    /// - The preview is an outline `width` pixels wide in its `color`,
    ///   inside `rect`, with its outer corners rounded by `radius`, over a
    ///   fill of the same color through which what is under it shows.
    /// - There is one preview. Showing it again moves it, and draws it
    ///   from scratch only when its rect or its style changed, or after
    ///   [`forget_drawn`](Self::forget_drawn).
    /// - It sits just behind `window`, in the same band of the z-order,
    ///   topmost or not: the window the user drags stays in front of it,
    ///   as does any window in front of that one. Showing it again puts
    ///   it back there, in front of a window that came between the two.
    /// - Like a border, it is only ever seen: it never takes focus or
    ///   input, never shows in the taskbar or the window switcher, and
    ///   showing, moving or hiding it never causes a
    ///   [`WindowEvent`](crate::WindowEvent).
    /// - A platform that cannot draw a preview returns
    ///   [`Error::Unsupported`](crate::Error::Unsupported), which is the
    ///   default. Windows land where they are dropped all the same.
    fn show_preview(&mut self, window: WindowId, rect: Rect, style: &PreviewStyle) -> Result<()> {
        let _ = (window, rect, style);
        Err(crate::Error::Unsupported("previewing where a window lands"))
    }

    /// Hides the drop preview, keeping it to show again. Does nothing when
    /// it is not shown, as the default does on a platform without one.
    fn hide_preview(&mut self) -> Result<()> {
        Ok(())
    }
}
