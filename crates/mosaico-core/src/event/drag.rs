//! What the platform sees as the user starts dragging a window.

use crate::Rect;

/// What [`WindowEvent::MoveStarted`](crate::WindowEvent::MoveStarted)
/// carries: what the platform saw as the drag started.
///
/// Each part is read when the platform sees the drag start, not when the
/// event is handled, which can be later: by then the window and the
/// pointer have moved on. A part the platform cannot read is `None`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DragStart {
    /// What the pointer holds the window by.
    pub grab: Option<Grab>,
    /// The window's visible bounds, as [`Desktop::rect`] reads them.
    ///
    /// [`Desktop::rect`]: crate::tiler::Desktop::rect
    pub rect: Option<Rect>,
    /// Whether the window is maximized, which a drag by its title bar
    /// soon ends.
    pub maximized: Option<bool>,
    /// Where the pointer is, in the coordinates of
    /// [`Desktop::cursor`](crate::tiler::Desktop::cursor).
    pub pointer: Option<(i32, i32)>,
}

/// What the pointer holds a window by as the user drags it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grab {
    /// The title bar, or another part of the window that moves it: the
    /// drag moves the window and keeps its size.
    TitleBar,
    /// Edges of the window's frame, two of them at a corner: the drag
    /// resizes the window from them.
    Edges(Edges),
}

/// Edges of a window's frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Edges {
    /// The left edge.
    pub left: bool,
    /// The top edge.
    pub top: bool,
    /// The right edge.
    pub right: bool,
    /// The bottom edge.
    pub bottom: bool,
}

#[cfg(test)]
impl crate::WindowEvent {
    /// The start of a drag of `window`, reported with nothing read, as a
    /// platform that can read none of it does.
    pub(crate) fn move_started(window: crate::WindowId) -> Self {
        let start = DragStart::default();
        Self::MoveStarted { window, start }
    }

    /// The end of a drag of `window`, reported with no pointer read.
    pub(crate) fn moved(window: crate::WindowId) -> Self {
        let pointer = None;
        Self::Moved { window, pointer }
    }
}
