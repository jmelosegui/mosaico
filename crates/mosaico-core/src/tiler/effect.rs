use crate::config::{Color, CornerStyle, HidingBehaviour};
use crate::{Rect, WindowId};

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
    },
    /// Move a window without waiting for its application: one that
    /// exists but has not been shown yet, or one just taken in while it
    /// is on screen, ahead of the layout pass. Must never block. The move
    /// may land later or not at all, so nothing in the step counts on it.
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
}

/// When a [`Effect::Place`] has to reach the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timing {
    /// Apply it right away.
    Now,
    /// The tiling manager keeps only the latest position per window and
    /// applies it once input goes quiet. Used for moves the user tends
    /// to repeat quickly, so a burst of key presses repaints once.
    ///
    /// Effects that follow for the same window ([`Effect::Border`],
    /// [`Effect::CenterCursor`]) must wait for the move.
    Coalesced,
}

/// How a focus border is drawn. Where the stroke goes is part of the
/// [`Decorations`](crate::Decorations) contract.
#[derive(Debug, Clone, PartialEq)]
pub struct BorderStyle {
    /// The window's visible bounds, which the border surrounds.
    pub rect: Rect,
    /// The stroke color. A configured value that is not a color is
    /// already replaced by
    /// [`BorderColors::FALLBACK`](crate::config::BorderColors::FALLBACK).
    pub color: Color,
    /// Stroke width in pixels.
    pub width: i32,
    /// Radius of the border's outer corners, in pixels.
    pub radius: i32,
}
