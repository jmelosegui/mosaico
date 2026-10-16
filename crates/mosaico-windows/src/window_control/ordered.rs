//! Making a window's other changes in order with the frames posted to
//! it: posted behind them while the frame table says something may still
//! be queued for the window, and made at once otherwise.
//!
//! A change that goes through here is noted in the table once made, so
//! the changes after it are posted behind it too; a change of show state
//! is noted with whether it leaves the window in view, which a placement
//! posted after it needs to know (see `posted_placement.rs`). A change that
//! fails is not noted.

use std::time::Instant;

use mosaico_core::{Rect, Result, WindowId};
use windows::Win32::Graphics::Gdi::REDRAW_WINDOW_FLAGS;

use crate::window::Window;

use super::Win32WindowControl;
use super::frames::Probe;
use super::hiding;

impl Win32WindowControl {
    /// Whether a change made to `window` at `now` is to be posted behind
    /// what may be queued for it. Read before the change is made, so the
    /// lock is not held while it is.
    pub(super) fn posts_behind_at(&self, window: WindowId, now: Instant) -> bool {
        self.frames().posts_behind(window, &Probe, now)
    }

    /// Makes `change` to `window` now, posted behind what may be queued
    /// for the window when told to, and notes what was posted.
    pub(super) fn ordered(
        &self,
        window: WindowId,
        change: impl FnOnce(bool) -> Result<()>,
    ) -> Result<()> {
        let now = Instant::now();
        let posted = self.posts_behind_at(window, now);
        change(posted)?;
        if posted {
            self.frames().posted(window, &Probe, now);
        }
        Ok(())
    }

    /// Makes `change` to `window`'s show state now, as `ordered` does,
    /// and notes whether it leaves the window `in_view`, which a
    /// placement posted after it needs to know.
    pub(super) fn ordered_show_state(
        &self,
        window: WindowId,
        in_view: bool,
        change: impl FnOnce(bool) -> Result<()>,
    ) -> Result<()> {
        let now = Instant::now();
        let posted = self.posts_behind_at(window, now);
        change(posted)?;
        let mut frames = self.frames();
        if posted {
            frames.posted_show_state(window, &Probe, now, in_view);
        } else {
            frames.show_state_made(window, &Probe);
        }
        Ok(())
    }

    /// Shows `window`, `win`, as `reveal` does once it has uncloaked it,
    /// and notes whether that leaves it in view.
    pub(super) fn show_revealed(&self, window: WindowId, win: Window) -> Result<()> {
        let in_view = hiding::revealed_in_view(win.is_minimized());
        self.ordered_show_state(window, in_view, |posted| hiding::show_revealed(win, posted))
    }

    /// Moves `window` so its visible bounds are `rect`: at once, or, when
    /// `posted`, behind what was posted to it (see
    /// `window_control/posted_placement.rs`).
    pub(super) fn move_into_place(
        &self,
        window: WindowId,
        rect: &Rect,
        posted: bool,
    ) -> Result<()> {
        if posted {
            let placed = self.post_placement(window, rect)?;
            let (first, border) = (placed.first, placed.border);
            mosaico_core::log_debug!("posted placement of {window}: {first:?}, {border:?}");
            Ok(())
        } else {
            Window::from_id(window).set_rect(rect)
        }
    }

    /// How `window` is repainted now: without waiting for its application
    /// to paint when something posted to it may still be queued, so that
    /// the paint comes after it.
    pub(super) fn repaint_flags(&self, window: WindowId) -> REDRAW_WINDOW_FLAGS {
        Window::repaint_flags(self.posts_behind_at(window, Instant::now()))
    }
}

#[cfg(test)]
#[path = "ordered_tests.rs"]
mod tests;
