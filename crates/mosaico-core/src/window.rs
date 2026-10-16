use crate::{Rect, Result};

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
