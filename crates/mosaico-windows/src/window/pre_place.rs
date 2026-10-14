//! Moving a window without waiting for its application.

use mosaico_core::{Rect, Result};
use windows::Win32::UI::WindowsAndMessaging::{
    SET_WINDOW_POS_FLAGS, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOSENDCHANGING,
    SWP_NOZORDER, SetWindowPos,
};

use crate::error::OrPlatform;
use crate::frame;

use super::Window;

/// Flags used when moving a window without waiting for its application.
///
/// `SWP_ASYNCWINDOWPOS` is the load bearing one: without it the call is
/// delivered to the owning thread synchronously and blocks until that
/// thread pumps its message queue, which some of these windows never do.
const PRE_PLACE_FLAGS: SET_WINDOW_POS_FLAGS = SET_WINDOW_POS_FLAGS(
    SWP_NOZORDER.0
        | SWP_NOACTIVATE.0
        | SWP_NOSENDCHANGING.0
        | SWP_NOCOPYBITS.0
        | SWP_ASYNCWINDOWPOS.0,
);

impl Window {
    /// Moves the window without waiting for the thread that owns it.
    ///
    /// `SetWindowPos` delivers WM_WINDOWPOSCHANGED, WM_MOVE and WM_SIZE to
    /// the owning thread and does not return until that thread has handled
    /// them, so it blocks for as long as that thread refuses to pump. That
    /// is fine for the layout pass, which moves real application windows
    /// that pump, but pre placement runs against windows that have been
    /// created and not yet shown, and some of those belong to threads that
    /// never pump at all. Crash handler watchers and similar helper windows
    /// are the common case. Blocking on one of those wedges the daemon
    /// permanently: no tiling, no hotkeys, no IPC. It also moves a window
    /// that has just been taken in while on screen, ahead of the layout
    /// pass, whose application may still be busy creating it. Waiting for
    /// that one would hold up every other window on the monitor.
    ///
    /// `SWP_ASYNCWINDOWPOS` makes the system post the request to the owning
    /// thread instead of sending it, so this returns immediately. The move
    /// is applied whenever that thread gets around to it, which is exactly
    /// the semantics pre placement wants, since nothing depends on the new
    /// position having landed: the layout pass still places the window.
    pub(crate) fn move_async(&self, rect: &Rect) -> Result<()> {
        // Compensate for invisible borders so the visible portion lands
        // exactly at the requested position and size, as set_rect does.
        let border = frame::border_offset(self.hwnd)?;
        let x = rect.x - border.left;
        let y = rect.y - border.top;
        let cx = rect.width + border.left + border.right;
        let cy = rect.height + border.top + border.bottom;

        mosaico_core::log_debug!(
            "move_async 0x{:X}: target({},{} {}x{})",
            self.hwnd.0 as usize,
            rect.x,
            rect.y,
            rect.width,
            rect.height
        );

        // SAFETY: SetWindowPos is safe to call with a valid HWND.
        // SWP_ASYNCWINDOWPOS posts the request rather than sending it, so
        // this cannot block on an unresponsive owning thread.
        unsafe { SetWindowPos(self.hwnd, None, x, y, cx, cy, PRE_PLACE_FLAGS) }
            .or_platform("move the window")
    }
}

#[cfg(test)]
#[path = "pre_place_tests.rs"]
mod tests;
