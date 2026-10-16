//! Reading where a window is, and posting it a frame, through Win32.

use mosaico_core::{Rect, Result};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowRect, GetWindowThreadProcessId, IsHungAppWindow, SET_WINDOW_POS_FLAGS,
    SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOSENDCHANGING, SWP_NOZORDER, SetWindowPos,
};

use crate::error::OrPlatform;
use crate::frame::BorderOffset;
use crate::monitor::rect_from_win32;
use crate::window::Window;

/// How a frame is posted.
///
/// `SWP_ASYNCWINDOWPOS` posts the move to the window's thread instead of
/// sending it, so a frame never waits for an application. `SWP_NOZORDER`
/// and `SWP_NOACTIVATE` leave the stacking order and focus alone.
/// `SWP_NOSENDCHANGING`, which `set_rect` uses too, keeps the application
/// from changing the frame, so its minimum size clamps no frame that the
/// final placement would not. `SWP_NOCOPYBITS` is left out: it would
/// throw the window's content away and make the application repaint all
/// of it on every frame of a plain move. So is `SWP_FRAMECHANGED`, which
/// `set_rect` adds for Chromium windows: Windows recalculates the client
/// area of a window whose size changes anyway, as it does while the user
/// resizes one, and the final placement still adds it and realigns the
/// GPU surface.
const FRAME_FLAGS: SET_WINDOW_POS_FLAGS = SET_WINDOW_POS_FLAGS(
    SWP_ASYNCWINDOWPOS.0 | SWP_NOZORDER.0 | SWP_NOACTIVATE.0 | SWP_NOSENDCHANGING.0,
);

/// The outer bounds of a window whose visible bounds are `rect`.
pub(super) fn outer(border: &BorderOffset, rect: &Rect) -> Rect {
    Rect::new(
        rect.x - border.left,
        rect.y - border.top,
        rect.width + border.left + border.right,
        rect.height + border.top + border.bottom,
    )
}

/// The visible bounds of a window whose outer bounds are `rect`.
pub(super) fn visible(border: &BorderOffset, rect: &Rect) -> Rect {
    Rect::new(
        rect.x + border.left,
        rect.y + border.top,
        rect.width - border.left - border.right,
        rect.height - border.top - border.bottom,
    )
}

/// Where the window is, invisible borders included, as `GetWindowRect`
/// reads it. `None` once the window is gone.
pub(super) fn outer_rect(win: Window) -> Option<Rect> {
    let mut rect = RECT::default();
    // SAFETY: GetWindowRect writes into a RECT on the stack, and fails
    // for a window that is gone.
    unsafe { GetWindowRect(win.hwnd(), &mut rect) }.ok()?;
    Some(rect_from_win32(&rect))
}

/// Whether the window's application handles its messages, as far as
/// Windows can tell: it does not after 5 seconds without doing so.
pub(super) fn responding(win: Window) -> bool {
    // SAFETY: IsHungAppWindow is a read-only query.
    !unsafe { IsHungAppWindow(win.hwnd()) }.as_bool()
}

/// Who owns the window: the thread that created it, with its process, so
/// a new window given the handle of an old one is told apart. `None` once
/// the window is gone.
pub(super) fn owner(win: Window) -> Option<u64> {
    let mut process = 0u32;
    // SAFETY: GetWindowThreadProcessId reads the owner of a window into a
    // u32 on the stack, and returns 0 for a window that is gone.
    let thread = unsafe { GetWindowThreadProcessId(win.hwnd(), Some(&mut process)) };
    (thread != 0).then(|| (u64::from(process) << 32) | u64::from(thread))
}

/// Posts the move of a window to the outer bounds `to`.
pub(super) fn post(win: Window, to: &Rect) -> Result<()> {
    let Rect {
        x,
        y,
        width,
        height,
    } = *to;
    // SAFETY: SWP_ASYNCWINDOWPOS posts the move to the window's thread
    // instead of sending it, so this returns at once, however long that
    // thread takes; a window gone in the meantime makes it fail.
    unsafe { SetWindowPos(win.hwnd(), None, x, y, width, height, FRAME_FLAGS) }
        .or_platform("move the window to a frame")
}

#[cfg(test)]
mod tests {
    use windows::Win32::UI::WindowsAndMessaging::SWP_NOCOPYBITS;

    use super::*;

    #[test]
    fn a_frame_is_posted_and_keeps_the_window_content() {
        // Arrange / Act / Assert -- posted, so that it never waits for an
        // application, and without SWP_NOCOPYBITS, which would make the
        // application repaint everything on every frame of a plain move.
        assert_ne!(FRAME_FLAGS.0 & SWP_ASYNCWINDOWPOS.0, 0);
        assert_ne!(FRAME_FLAGS.0 & SWP_NOSENDCHANGING.0, 0);
        assert_eq!(FRAME_FLAGS.0 & SWP_NOCOPYBITS.0, 0);
    }

    #[test]
    fn the_outer_bounds_of_a_frame_give_its_visible_bounds_back() {
        // Arrange -- the invisible resize borders of a Windows 11 window.
        let border = BorderOffset {
            left: 7,
            top: 0,
            right: 7,
            bottom: 7,
        };
        let rect = Rect::new(40, 30, 120, 90);

        // Act
        let around = outer(&border, &rect);

        // Assert
        assert_eq!(around, Rect::new(33, 30, 134, 97));
        assert_eq!(visible(&border, &around), rect);
    }
}
