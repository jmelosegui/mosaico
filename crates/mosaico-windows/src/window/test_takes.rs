//! How the windows of the tests' own take the moves made to them: some
//! put them back or round them, as some applications do.

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, GetWindowRect, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    SetWindowPos, WINDOWPOS, WM_WINDOWPOSCHANGED, WNDPROC,
};

/// How a window of the tests' own takes the moves made to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Takes {
    /// Where it is moved.
    Moves,
    /// Back where it was made, at once, as a window sent back where it was.
    PutsBack,
    /// With its size rounded down to a multiple of 16, as a terminal
    /// snapping to its cells.
    RoundsSize,
}

impl Takes {
    /// The window class of such a window, and its procedure.
    pub(super) fn class(self, default: WNDPROC) -> (&'static str, WNDPROC) {
        match self {
            Self::Moves => ("MosaicoHiddenTestWindow\0", default),
            Self::PutsBack => ("MosaicoPutsBackTestWindow\0", Some(puts_back_proc)),
            Self::RoundsSize => ("MosaicoRoundsSizeTestWindow\0", Some(rounds_size_proc)),
        }
    }
}

/// A window that puts itself back at 0, 0, 1 by 1, where it was made, as
/// soon as it is moved.
unsafe extern "system" fn puts_back_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let mut rect = RECT::default();
    // SAFETY: reads and moves a window of the calling thread, which owns
    // it, and then hands the message to the default handler.
    unsafe {
        if msg == WM_WINDOWPOSCHANGED
            && GetWindowRect(hwnd, &mut rect).is_ok()
            && (rect.left, rect.top, rect.right, rect.bottom) != (0, 0, 1, 1)
        {
            let _ = SetWindowPos(hwnd, None, 0, 0, 1, 1, SWP_NOZORDER | SWP_NOACTIVATE);
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

/// A window that rounds its size down to a multiple of 16 whenever it is
/// resized to another.
unsafe extern "system" fn rounds_size_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: for WM_WINDOWPOSCHANGED, `lparam` points to the window's
    // new position; the window belongs to the calling thread.
    unsafe {
        if msg == WM_WINDOWPOSCHANGED {
            let pos = &*(lparam.0 as *const WINDOWPOS);
            let (cx, cy) = (pos.cx - pos.cx % 16, pos.cy - pos.cy % 16);
            let sized = pos.flags.0 & SWP_NOSIZE.0 == 0;
            if sized && cx > 0 && cy > 0 && (cx, cy) != (pos.cx, pos.cy) {
                let flags = SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE;
                let _ = SetWindowPos(hwnd, None, 0, 0, cx, cy, flags);
            }
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}
