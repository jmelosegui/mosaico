//! What posting a window's placement behind what was posted to it reads
//! of the window and does to it. What it decides from that is in
//! `window_control/posted_placement.rs`.

use mosaico_core::{Rect, Result};
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_STYLE, GetWindowLongW, SW_SHOWNOACTIVATE, ShowWindowAsync, WS_VISIBLE,
};

use crate::error::OrPlatform;
use crate::frame::BorderOffset;

use super::Window;

impl Window {
    /// Whether the window has `WS_VISIBLE`, which hiding it takes away and
    /// showing it gives back, and which a cloaked window keeps, as one
    /// inside a hidden window does. A read that does not wait for the
    /// application.
    pub(crate) fn is_shown(&self) -> bool {
        // SAFETY: GetWindowLongW is a read-only query; a window destroyed
        // in the meantime reads as 0.
        let style = unsafe { GetWindowLongW(self.hwnd, GWL_STYLE) } as u32;
        style & WS_VISIBLE.0 != 0
    }

    /// Posts `SW_SHOWNOACTIVATE` to the window's thread, behind what was
    /// posted to it before: it restores a maximized or minimized window,
    /// shows a hidden one, and leaves a shown one in its normal state as
    /// it is, without activating it.
    pub(crate) fn post_restore(&self) -> Result<()> {
        // SAFETY: ShowWindowAsync only posts the show state to the
        // window's thread; an invalid handle makes it fail without side
        // effects.
        unsafe { ShowWindowAsync(self.hwnd, SW_SHOWNOACTIVATE) }
            .ok()
            .or_platform("queue the restore of the window")
    }

    /// Posts the move of the window to its thread, behind what was posted
    /// to it before, so that inside its invisible borders `border` its
    /// visible bounds are `rect`, with the flags and the Chromium rule of
    /// `set_rect`.
    pub(crate) fn post_move(&self, rect: &Rect, border: &BorderOffset) -> Result<()> {
        self.move_to(rect, border, true)
    }
}
