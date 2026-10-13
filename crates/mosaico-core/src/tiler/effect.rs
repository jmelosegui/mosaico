use crate::config::{CornerStyle, HidingBehaviour};
use crate::{Rect, WindowId};

/// A change the tiler wants made to the desktop.
///
/// [`Tiler::step`](super::Tiler::step) returns these in the order they
/// must be applied. The platform applies them and records what it
/// expects the OS to report back in an
/// [`Expectations`](super::Expectations), so that the window events our
/// own changes cause are not mistaken for user actions.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Move and resize a window, then repaint it.
    Place {
        /// The window.
        window: WindowId,
        /// Where its visible bounds go.
        rect: Rect,
        /// Whether the platform may hold the move back. See [`Timing`].
        timing: Timing,
    },
    /// Move a window that exists but has not been shown yet, without
    /// waiting for its owner to respond. Must never block.
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
    /// The platform may hold focus changes back while the user is still
    /// navigating, and only apply the last one. The tiler has already
    /// updated its own idea of focus, borders and the bar.
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
    /// Apply the configured corner style to a managed window's frame.
    Decorate {
        /// The window.
        window: WindowId,
        /// The corner style.
        corners: CornerStyle,
    },
    /// Undo [`Effect::Decorate`] on a window that is no longer managed.
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
    /// The platform may keep only the latest position per window and
    /// apply it once input goes quiet. Used for moves the user tends to
    /// repeat quickly, so a burst of key presses repaints once.
    ///
    /// Effects that follow for the same window ([`Effect::Border`],
    /// [`Effect::CenterCursor`]) must wait for the move.
    Coalesced,
}

/// How a focus border is drawn.
#[derive(Debug, Clone, PartialEq)]
pub struct BorderStyle {
    /// The window's visible bounds.
    pub rect: Rect,
    /// Hex color, such as `"#00b4d8"`.
    pub color: String,
    /// Stroke width in pixels.
    pub width: i32,
    /// Corner radius in pixels.
    pub radius: i32,
}
