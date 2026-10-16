//! The frames of transitions, posted without waiting.
//!
//! What each frame does is decided by the core [`FrameTable`]. This file
//! is how the table reads Win32 windows and posts their moves: a frame is
//! posted with `SetWindowPos` and `SWP_ASYNCWINDOWPOS`, like `pre_place`,
//! so a window whose thread is slow or hung never holds the daemon up,
//! and where a window is is read with `GetWindowRect`, in outer bounds.
//! The invisible borders around a window, which take a DWM query to read,
//! with the DPI of the display they are drawn at, and whether Windows maps
//! the moves made to it to pixels of its own, are read with the first
//! frame of its motion. A window whose
//! application is not responding (`IsHungAppWindow`) gets nothing posted.
//!
//! A thread handles what is posted to it in order, so the other changes
//! made to a window while something may still be queued for it are
//! posted too, behind it, as the table says (see `ordered.rs`).
//!
//! [`FrameTable`]: mosaico_core::frames::FrameTable

#[path = "frames_dpi.rs"]
mod dpi;
#[path = "frames_kept.rs"]
mod kept;
#[path = "frames_post.rs"]
mod posting;

use std::time::Instant;

use mosaico_core::frames::{Frame, FrameControl, FrameOutcome, FrameProbe};
use mosaico_core::{Rect, Result, WindowId};

use crate::frame;
use crate::window::Window;

use super::Win32WindowControl;
pub(super) use kept::Kept;
use posting::{outer, outer_rect, owner, post, responding, visible};

impl FrameControl for Win32WindowControl {
    fn place_frames(&self, frames: &[Frame], outcomes: &mut Vec<FrameOutcome>) -> Result<()> {
        self.frames()
            .place(frames, &mut Probe, Instant::now(), outcomes);
        Ok(())
    }

    fn posts_behind(&self, window: WindowId) -> bool {
        self.posts_behind_at(window, Instant::now())
    }

    fn settle_responding(&self) {
        self.frames().settle_responding(&Probe, Instant::now());
    }
}

/// Win32 windows, as the frame table reads and moves them.
pub(super) struct Probe;

impl FrameProbe for Probe {
    type Kept = Kept;

    fn exists(&self, window: WindowId) -> bool {
        Window::from_id(window).is_valid()
    }

    fn minimized(&self, window: WindowId) -> bool {
        Window::from_id(window).is_minimized()
    }

    fn maximized(&self, window: WindowId) -> bool {
        Window::from_id(window).is_maximized()
    }

    fn shown(&self, window: WindowId) -> bool {
        Window::from_id(window).is_shown()
    }

    fn responding(&self, window: WindowId) -> bool {
        responding(Window::from_id(window))
    }

    fn owner(&self, window: WindowId) -> Option<u64> {
        owner(Window::from_id(window))
    }

    fn at(&self, window: WindowId) -> Option<Rect> {
        outer_rect(Window::from_id(window))
    }

    fn keep(&self, window: WindowId) -> Option<Kept> {
        let win = Window::from_id(window);
        let border = frame::border_offset(win.hwnd()).ok()?;
        let (scaled, dpi) = (dpi::scaled(win), dpi::display_dpi(win));
        Some(Kept {
            border,
            scaled,
            dpi,
        })
    }

    fn aim(&self, window: WindowId, kept: &Kept, rect: Rect) -> Rect {
        let to = outer(&kept.border, &rect);
        if kept.scaled {
            dpi::reachable(Window::from_id(window), to)
        } else {
            to
        }
    }

    fn visible(&self, kept: &Kept, at: Rect) -> Rect {
        visible(&kept.border, &at)
    }

    /// A frame the system refuses holds nothing back: the next one is
    /// posted at once.
    fn post(&mut self, window: WindowId, to: Rect) -> bool {
        post(Window::from_id(window), &to).is_ok()
    }
}

#[cfg(test)]
#[path = "frames_tests.rs"]
mod tests;
