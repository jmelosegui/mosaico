//! A placement posted behind what was posted to a window: how it ends a
//! maximized state, and the invisible borders it moves the window with.
//!
//! The move lands after everything queued before it, so it has to land
//! on a window in its normal state, and with the borders of that state.
//! The decisions are plain functions of what is read of the window, so
//! that each is tested on its own; the reads never wait for the
//! application.

use mosaico_core::frames::FrameProbe;
use mosaico_core::{Rect, Result, WindowId};

use crate::frame::{self, BorderOffset};
use crate::window::Window;

use super::Win32WindowControl;
use super::frames::Probe;

/// What a posted placement goes by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Seen {
    /// Whether the window is maximized now (`IsZoomed`).
    pub(super) maximized: bool,
    /// Whether it is minimized now (`IsIconic`).
    pub(super) minimized: bool,
    /// Whether it is to be in view once what was posted lands, as the
    /// last change of its show state posted says, if that is still ahead.
    pub(super) in_view: Option<bool>,
    /// Whether it has `WS_VISIBLE` now.
    pub(super) shown: bool,
    /// Whether its application responds (`IsHungAppWindow`).
    pub(super) responding: bool,
}

/// What a posted placement does to the window before its move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum First {
    /// Nothing.
    Nothing,
    /// Post `SW_SHOWNOACTIVATE`, which ends a maximized state.
    Restore,
    /// Clear `WS_MAXIMIZE` at once, which waits for the application.
    ClearMaximized,
}

/// What a placement posted to a window seen as `seen` does first.
///
/// A maximized window is restored when it is to be in view once what was
/// posted lands, which a cloaked window is, as one with `WS_VISIBLE` and
/// nothing posted to say otherwise. One out of view would be shown by the
/// restore, so it has its style cleared at once instead, once it no longer
/// has `WS_VISIBLE`: before that, the hide or minimize queued to take it
/// out of view could land between the read of its style and the write,
/// which would undo it. Nor is the style cleared while the application
/// does not respond, which the clear would wait for. Either window stays
/// maximized, and a later show restores its old normal bounds. A
/// minimized window that a show queued ahead brings back in view is
/// restored again behind that show, which ends the maximized state it may
/// come back in, and does nothing to one it brings back in its normal
/// state; never one that is to stay out of view, which it would show.
pub(super) fn first(seen: &Seen) -> First {
    if seen.maximized {
        if seen.in_view.unwrap_or(seen.shown) {
            First::Restore
        } else if seen.responding && !seen.shown {
            First::ClearMaximized
        } else {
            First::Nothing
        }
    } else if seen.minimized && seen.in_view == Some(true) {
        First::Restore
    } else {
        First::Nothing
    }
}

/// The invisible borders a posted move is made with, out of `kept`, the
/// ones read with the window's frames in its normal state: those for a
/// window minimized or maximized now, which is in its normal state once
/// the move lands, and `None`, to read them now, for any other, or when
/// none were kept.
pub(super) fn kept_border(normal: bool, kept: Option<BorderOffset>) -> Option<BorderOffset> {
    kept.filter(|_| !normal)
}

/// What a posted placement did before its move, and the invisible borders
/// it moved the window with.
#[derive(Debug, Clone, Copy)]
pub(super) struct Placed {
    pub(super) first: First,
    pub(super) border: BorderOffset,
}

impl Win32WindowControl {
    /// Posts the placement of `window` to `rect` behind what was posted to
    /// it, as [`first`] and [`kept_border`] decide from what the table
    /// keeps for it and what is read of it now, and says what it did.
    pub(super) fn post_placement(&self, window: WindowId, rect: &Rect) -> Result<Placed> {
        let win = Window::from_id(window);
        let (kept, in_view) = {
            let mut frames = self.frames();
            let kept = frames
                .kept_for(window, &Probe)
                .map(|kept| kept.border_for(win));
            (kept, frames.in_view_once_landed(window, &Probe))
        };
        let (maximized, minimized) = (win.is_maximized(), win.is_minimized());
        let seen = Seen {
            maximized,
            minimized,
            in_view,
            shown: win.is_shown(),
            responding: Probe.responding(window),
        };
        let first = first(&seen);
        match first {
            First::Restore => win.post_restore()?,
            First::ClearMaximized => win.clear_maximized(),
            First::Nothing => {}
        }
        let border = match kept_border(!maximized && !minimized, kept) {
            Some(border) => border,
            None => frame::border_offset(win.hwnd())?,
        };
        win.post_move(rect, &border)?;
        Ok(Placed { first, border })
    }
}

#[cfg(test)]
#[path = "posted_placement_border_tests.rs"]
mod border_tests;
#[cfg(test)]
#[path = "posted_placement_call_tests.rs"]
mod call_tests;
#[cfg(test)]
#[path = "posted_placement_tests.rs"]
mod tests;
