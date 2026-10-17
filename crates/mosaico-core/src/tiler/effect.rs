use crate::config::{CornerStyle, HidingBehaviour};
use crate::{Rect, WindowId};

use super::{BorderStyle, PreviewStyle, Timing, Transition};

/// A change the tiler wants made to the desktop.
///
/// [`Tiler::step`](super::Tiler::step) returns these in the order they
/// must be applied. The [`TilingManager`](super::TilingManager) applies
/// them through the platform and records what it expects the OS to
/// report back in an [`Expectations`](super::Expectations), so that the
/// window events our own changes cause are not mistaken for user
/// actions.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Move and resize a window, then repaint it.
    Place {
        /// The window.
        window: WindowId,
        /// Where its visible bounds go.
        rect: Rect,
        /// Whether the move may be held back. See [`Timing`].
        timing: Timing,
        /// Whether it may move there over time. See [`Transition`].
        transition: Transition,
    },
    /// Move a window without waiting for its application: one that
    /// exists but has not been shown yet, or one just taken in while it
    /// is on screen, ahead of the layout pass. Must never block. The move
    /// may land at once, later or not at all, so nothing in the step
    /// counts on it.
    PrePlace {
        /// The window.
        window: WindowId,
        /// Where its visible bounds go.
        rect: Rect,
    },
    /// Hide a window whose workspace was switched away from.
    Hide {
        /// The window.
        window: WindowId,
        /// How to hide it.
        how: HidingBehaviour,
    },
    /// Show a window hidden by [`Effect::Hide`] again.
    Show {
        /// The window.
        window: WindowId,
        /// How it was hidden.
        how: HidingBehaviour,
    },
    /// Make a window visible no matter how it was hidden. Used when the
    /// tiler shuts down, so nothing is left invisible.
    Reveal {
        /// The window.
        window: WindowId,
    },
    /// Give a window keyboard focus.
    ///
    /// The tiling manager holds focus changes back while the user is
    /// still navigating, and only applies the last one. The tiler has
    /// already updated its own idea of focus, borders and the bar.
    Focus {
        /// The window.
        window: WindowId,
        /// Whether to put the pointer on the window afterwards.
        move_cursor: bool,
    },
    /// Put the pointer at the center of a window, once it is in place.
    CenterCursor {
        /// The window.
        window: WindowId,
    },
    /// Ask a window to close, letting the application save its state.
    Close {
        /// The window.
        window: WindowId,
    },
    /// Minimize a window.
    Minimize {
        /// The window.
        window: WindowId,
    },
    /// Repaint a window without moving it.
    Repaint {
        /// The window.
        window: WindowId,
    },
    /// Put a window directly behind another, without focusing either, so
    /// the other stays in front where they overlap.
    PutBehind {
        /// The window to put behind.
        window: WindowId,
        /// The window it goes behind.
        other: WindowId,
    },
    /// Stop tiling a window that left the layout. Nothing changes on
    /// screen: the tiling manager drops what it still holds back for the
    /// window, since that was decided while the window was tiled.
    Untile {
        /// The window.
        window: WindowId,
    },
    /// Apply the configured corner style to a managed window's frame.
    Decorate {
        /// The window.
        window: WindowId,
        /// The corner style.
        corners: CornerStyle,
    },
    /// Undo [`Effect::Decorate`] on a window that is no longer managed.
    /// Only ever asked for a window that was decorated.
    Undecorate {
        /// The window.
        window: WindowId,
    },
    /// Draw, move or hide the focus border around a window.
    Border {
        /// The window the border surrounds.
        window: WindowId,
        /// How to draw it, or `None` to hide it.
        style: Option<BorderStyle>,
    },
    /// Destroy the border around a window that left the screen.
    DropBorder {
        /// The window the border surrounded.
        window: WindowId,
    },
    /// Draw each border from scratch the next time it is drawn, even
    /// one that would look as it does now.
    ///
    /// A platform may keep what it drew of a border and only move it
    /// when the border is drawn again the same way, and what it kept can
    /// be lost outside mosaico, as when the display changes. The tiler
    /// asks for this ahead of the borders of a retile the user asks for
    /// and of a display change that changes anything.
    RedrawBorders,
    /// Show the drop preview where the window the user drags would land,
    /// or move it there.
    ShowPreview {
        /// The window the user drags, which stays in front of it.
        window: WindowId,
        /// The visible bounds the window would get.
        rect: Rect,
        /// How to draw it.
        style: PreviewStyle,
    },
    /// Hide the drop preview.
    HidePreview,
}
