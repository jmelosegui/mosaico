//! Putting a window behind another, through Win32.
//!
//! `SetWindowPos` reorders a window after the one it is given as the
//! window to insert after, and moves it into that window's band of the
//! z-order: a window inserted after a topmost one (`WS_EX_TOPMOST`)
//! becomes topmost, and a topmost one inserted after one that is not
//! stops being topmost. So a window is only reordered against one of its
//! own band, as [`Behind`] says. A window that is not topmost is behind
//! every topmost one already. Bringing it to the front of its own band
//! instead, with `HWND_TOP`, does nothing unless its process may take
//! the foreground, which an application in the background may not.
//!
//! The bands are read just before the call, and Windows has no call that
//! reads and reorders at once: another process can pin or unpin either
//! window in between, and a window that closes in between reads as not
//! topmost. Then the window may change band after all, as it would have
//! without the check.

use mosaico_core::Result;
use mosaico_core::window::Behind;
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongPtrW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos,
    WS_EX_TOPMOST,
};

use crate::error::OrPlatform;
use crate::window::Window;

/// Puts `window` directly behind `other` when both are in the same band,
/// and leaves it where it is otherwise. `SWP_NOMOVE`, `SWP_NOSIZE` and
/// `SWP_NOACTIVATE` leave the position and size of `window` and the
/// foreground window alone.
pub(super) fn put_behind(window: Window, other: Window) -> Result<()> {
    if Behind::new(is_topmost(window), is_topmost(other)) == Behind::Stay {
        return Ok(());
    }
    // SAFETY: SetWindowPos only reorders `window` here, after `other`,
    // which both exist; a window destroyed in the meantime makes it fail.
    unsafe {
        SetWindowPos(
            window.hwnd(),
            Some(other.hwnd()),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
    }
    .or_platform("put the window behind another")
}

/// Whether a window is in the topmost band of the z-order.
fn is_topmost(window: Window) -> bool {
    // SAFETY: GetWindowLongPtrW is a read-only query; a window destroyed
    // in the meantime reads as 0, so as not topmost.
    let ex_style = unsafe { GetWindowLongPtrW(window.hwnd(), GWL_EXSTYLE) } as u32;
    ex_style & WS_EX_TOPMOST.0 != 0
}

#[cfg(test)]
#[path = "stacking_tests.rs"]
mod tests;
