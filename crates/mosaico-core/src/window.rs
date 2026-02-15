use crate::Rect;

/// Platform-agnostic window trait.
///
/// Each platform crate (e.g. `mosaico-windows`) provides its own implementation.
pub trait Window {
    /// Returns the window title.
    fn title(&self) -> String;

    /// Returns the window class name.
    fn class(&self) -> String;

    /// Returns the window bounding rectangle.
    fn rect(&self) -> Rect;

    /// Returns whether the window is currently visible.
    fn is_visible(&self) -> bool;
}
