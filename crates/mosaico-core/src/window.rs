use crate::Rect;

/// A boxed error type for window operations.
///
/// `Box<dyn std::error::Error>` is Rust's equivalent of C#'s base `Exception`.
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

    /// Returns whether the window is currently visible.
    fn is_visible(&self) -> bool;
}
