use crate::Rect;

/// A boxed error type for window operations.
///
/// Any error type that implements the `Error` trait can be boxed into this.
/// We'll replace this with a custom error type as the project matures.
pub type WindowResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Platform-agnostic window trait.
///
/// Each platform crate (e.g. `mosaico-windows`) provides its own implementation.
pub trait Window {
    /// Returns the window title.
    fn title(&self) -> WindowResult<String>;

    /// Returns the window class name.
    fn class(&self) -> WindowResult<String>;

    /// Returns the window bounding rectangle.
    fn rect(&self) -> WindowResult<Rect>;

    /// Moves and resizes the window to the given rectangle.
    ///
    /// Takes `&self` because the mutation happens on the OS side via the
    /// window handle — the Rust struct itself doesn't change.
    fn set_rect(&self, rect: &Rect) -> WindowResult<()>;

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
