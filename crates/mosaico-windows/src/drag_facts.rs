//! What the start and the end of a drag carry on Win32, read in the hook
//! as it reports them.
//!
//! - The pointer is where `GetCursorPos` has it, the window's rect its
//!   visible bounds, as the desktop reads them, and the window is
//!   maximized when `IsZoomed` says so.
//! - What the pointer holds the window by is what the window answers to
//!   `WM_NCHITTEST` at the pointer: its caption, or a sizing edge or
//!   corner of its frame. Anything else, such as its client area for a
//!   drag the application started itself, holds nothing the tiler can
//!   tell. The window moves or resizes with the pointer as it is dragged,
//!   so the part under the pointer is still the part it was grabbed by
//!   when the hook reports the start a little late.
//! - The point is mapped to the window's own coordinates first, as a
//!   window that is not aware of the display's DPI works in those.
//! - The window is asked with `SendMessageTimeoutW`. `SMTO_BLOCK` keeps
//!   the event thread from handling anything else meanwhile, so no
//!   report overtakes this one, and a window that does not answer within
//!   [`HIT_TEST_TIMEOUT_MS`], or is not responding, holds nothing.

use mosaico_core::WindowEvent;
use mosaico_core::event::{DragStart, Edges, Grab};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::UI::HiDpi::PhysicalToLogicalPointForPerMonitorDPI;
use windows::Win32::UI::WindowsAndMessaging::{
    HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT, HTCAPTION, HTLEFT, HTRIGHT, HTTOP, HTTOPLEFT,
    HTTOPRIGHT, SMTO_ABORTIFHUNG, SMTO_BLOCK, SendMessageTimeoutW, WM_NCHITTEST,
};

use crate::window::Window;

/// How long a window has to say what is under the pointer as its drag
/// starts. One that responds answers in microseconds.
const HIT_TEST_TIMEOUT_MS: u32 = 50;

/// `event` with what the platform reads as it reports the start or the
/// end of a drag of `hwnd`. Any other event is returned as it is.
pub(crate) fn read(event: WindowEvent, hwnd: HWND) -> WindowEvent {
    match event {
        WindowEvent::MoveStarted { window, .. } => {
            let pointer = crate::pointer::position().ok();
            let win = Window::new(hwnd);
            let start = DragStart {
                grab: pointer.and_then(|at| hit_test(hwnd, at)).and_then(grab_of),
                rect: win.rect().ok(),
                maximized: Some(win.is_maximized()),
                pointer,
            };
            WindowEvent::MoveStarted { window, start }
        }
        WindowEvent::Moved { window, .. } => WindowEvent::Moved {
            window,
            pointer: crate::pointer::position().ok(),
        },
        other => other,
    }
}

/// What the pointer holds a window by, from the code the window answers
/// `WM_NCHITTEST` with.
pub(crate) fn grab_of(code: u32) -> Option<Grab> {
    let edges = |left, top, right, bottom| {
        Some(Grab::Edges(Edges {
            left,
            top,
            right,
            bottom,
        }))
    };
    match code {
        HTCAPTION => Some(Grab::TitleBar),
        HTLEFT => edges(true, false, false, false),
        HTRIGHT => edges(false, false, true, false),
        HTTOP => edges(false, true, false, false),
        HTBOTTOM => edges(false, false, false, true),
        HTTOPLEFT => edges(true, true, false, false),
        HTTOPRIGHT => edges(false, true, true, false),
        HTBOTTOMLEFT => edges(true, false, false, true),
        HTBOTTOMRIGHT => edges(false, false, true, true),
        _ => None,
    }
}

/// The point `(x, y)` packed as a mouse message carries it: `x` in the
/// low word and `y` in the high one, each as a signed 16-bit number.
pub(crate) fn point_param(x: i32, y: i32) -> LPARAM {
    let packed = (u32::from(y as u16) << 16) | u32::from(x as u16);
    LPARAM(packed as isize)
}

/// What `hwnd` says is under the pointer at `at`, or `None` when it does
/// not say in time.
fn hit_test(hwnd: HWND, at: (i32, i32)) -> Option<u32> {
    let mut point = POINT { x: at.0, y: at.1 };
    let mut code = 0usize;
    // SAFETY: the conversion only reads the DPI of the window and writes
    // the point on the stack, which it leaves physical when it fails.
    // WM_NCHITTEST only asks the window what is at the point, and the
    // answer is written to `code` on the stack. The call returns within
    // the timeout whatever the window does.
    let sent = unsafe {
        let _ = PhysicalToLogicalPointForPerMonitorDPI(Some(hwnd), &mut point);
        SendMessageTimeoutW(
            hwnd,
            WM_NCHITTEST,
            WPARAM(0),
            point_param(point.x, point.y),
            SMTO_BLOCK | SMTO_ABORTIFHUNG,
            HIT_TEST_TIMEOUT_MS,
            Some(&mut code),
        )
    };
    (sent.0 != 0).then_some(code as u32)
}

#[cfg(test)]
#[path = "drag_facts_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "drag_facts_hit_tests.rs"]
mod hit_tests;

#[cfg(test)]
#[path = "drag_facts_read_tests.rs"]
mod read_tests;
