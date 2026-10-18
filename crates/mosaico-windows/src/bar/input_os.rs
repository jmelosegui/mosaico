//! The calls a bar's window makes for the pointer: holding it through a
//! press, hearing when it leaves, where it is, its shape, and letting it
//! through to the windows under the bar.

use mosaico_core::Result;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    ReleaseCapture, SetCapture, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetCursorPos, GetWindowLongPtrW, IDC_ARROW, IDC_HAND, LoadCursorW, SetCursor,
    SetWindowLongPtrW, WS_EX_TRANSPARENT,
};

use crate::error::OrPlatform;

/// Has `hwnd` get the pointer's input until [`let_go`], wherever the
/// pointer goes while the button pressed over it is held.
pub(super) fn hold(hwnd: HWND) {
    // SAFETY: SetCapture takes the pointer for a window of this thread.
    // It returns the window that had it, which nothing here needs.
    unsafe {
        SetCapture(hwnd);
    }
}

/// Gives back the pointer [`hold`] took.
pub(super) fn let_go() {
    // SAFETY: ReleaseCapture releases this thread's hold on the pointer.
    // It only fails when there is none, which leaves nothing to do.
    unsafe {
        let _ = ReleaseCapture();
    }
}

/// Asks to be told when the pointer leaves `hwnd`, with one
/// `WM_MOUSELEAVE`. Says whether it was asked.
pub(super) fn hear_leaving(hwnd: HWND) -> bool {
    let mut track = TRACKMOUSEEVENT {
        cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
        dwFlags: TME_LEAVE,
        hwndTrack: hwnd,
        dwHoverTime: 0,
    };
    // SAFETY: TrackMouseEvent reads the stack allocated request, for a
    // window of this thread.
    unsafe { TrackMouseEvent(&mut track) }.is_ok()
}

/// `point`, in screen coordinates, in the coordinates of `hwnd`: pixels
/// from its top left corner.
pub(super) fn on_window(hwnd: HWND, (x, y): (i32, i32)) -> (i32, i32) {
    let mut point = POINT { x, y };
    // SAFETY: ScreenToClient converts the stack allocated point in place.
    // It only fails for a window that is gone, which leaves it as it was.
    unsafe {
        let _ = ScreenToClient(hwnd, &mut point);
    }
    (point.x, point.y)
}

/// Where the pointer is, in the coordinates of `hwnd`, if it can be read.
pub(super) fn pointer_on(hwnd: HWND) -> Option<(i32, i32)> {
    let mut point = POINT::default();
    // SAFETY: GetCursorPos writes the pointer position into a stack
    // allocated POINT and changes nothing.
    unsafe { GetCursorPos(&mut point) }.ok()?;
    Some(on_window(hwnd, (point.x, point.y)))
}

/// Shows the hand, over something to click, or the arrow.
pub(super) fn show_pointer(hand: bool) {
    let shape = if hand { IDC_HAND } else { IDC_ARROW };
    // SAFETY: LoadCursorW gives one of the system's shared cursors, which
    // are never destroyed, and SetCursor shows it while the pointer is
    // over the window whose WM_SETCURSOR is being handled.
    unsafe {
        if let Ok(cursor) = LoadCursorW(None, shape) {
            SetCursor(Some(cursor));
        }
    }
}

/// Has `hwnd` let the pointer through to the windows under it, or take
/// it, with or without `WS_EX_TRANSPARENT`.
pub(super) fn let_through(hwnd: HWND, through: bool) -> Result<()> {
    let flag = WS_EX_TRANSPARENT.0 as isize;
    // SAFETY: reads and writes the extended style of a window this thread
    // made. Its style is never 0, as it is layered, so 0 back from
    // SetWindowLongPtrW means it failed.
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let wanted = if through { style | flag } else { style & !flag };
        if wanted != style && SetWindowLongPtrW(hwnd, GWL_EXSTYLE, wanted) == 0 {
            return Err(windows::core::Error::from_win32())
                .or_platform("switch the bar between taking the pointer and letting it through");
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "input_support.rs"]
mod input_support;

#[cfg(test)]
#[path = "input_tests.rs"]
mod input_tests;

#[cfg(test)]
#[path = "input_wheel_tests.rs"]
mod input_wheel_tests;
