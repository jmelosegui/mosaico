//! What is kept about a window for its motion, and its invisible borders
//! at the DPI of the display it is on when a placement is posted.
//!
//! A window that Windows stretches, one that is not per-monitor DPI
//! aware, has its borders scaled with it. Any other has its frame drawn at
//! its DPI, wider by what the frame metrics say. That holds for a window
//! per-monitor aware of the second version, and is assumed for one of the
//! first, whose frame Windows scales only when its application asks it
//! to: one that does not keeps its borders, and lands off by the
//! difference.

use crate::frame::BorderOffset;
use crate::window::Window;

use super::dpi::{self, FrameWidths};

/// What is kept about a window for its motion, read with its first frame.
#[derive(Debug, Clone, Copy)]
pub(in crate::window_control) struct Kept {
    /// Its invisible borders.
    pub(in crate::window_control) border: BorderOffset,
    /// Whether Windows maps the moves made to it to pixels of its own.
    pub(in crate::window_control) scaled: bool,
    /// The DPI of its display then, which its borders are drawn at.
    pub(in crate::window_control) dpi: Option<i32>,
}

impl Kept {
    /// Its invisible borders, drawn at the DPI of the display `win` is on
    /// now (see [`border_at`](Self::border_at)).
    pub(in crate::window_control) fn border_for(&self, win: Window) -> BorderOffset {
        self.border_at(dpi::display_dpi(win), dpi::frame_widths)
    }

    /// Its invisible borders at the DPI `now`: as kept while that is the
    /// DPI they were read at, or either is not known. At another, a window
    /// Windows stretches has them scaled by the ratio of the two, and any
    /// other, whose frame Windows draws at the window's DPI, has each side
    /// moved by how much wider `widths` says that frame is now than then.
    pub(super) fn border_at(
        &self,
        now: Option<i32>,
        widths: impl Fn(i32) -> Option<FrameWidths>,
    ) -> BorderOffset {
        let (Some(then), Some(now)) = (self.dpi, now) else {
            return self.border;
        };
        if then == now {
            self.border
        } else if self.scaled {
            dpi::scale_border(&self.border, now, then)
        } else if let (Some(then), Some(now)) = (widths(then), widths(now)) {
            dpi::shift_border(&self.border, then, now)
        } else {
            self.border
        }
    }
}
