//! Where the pointer grabs a window, for the drag and drop tests: by its
//! title bar, a point the window itself says is its caption, as it answers
//! `WM_NCHITTEST`, below the band at its top that resizes it; or by its
//! right edge, a point it says is that edge.

use mosaico_core::{Rect, WindowId};

use super::win32;

/// How far below the top of a window's visible bounds the pointer first
/// tries to grab its caption, and how far down it looks for it, in pixels
/// at 96 DPI.
const FIRST_TRY: i32 = 12;
const DEEPEST: i32 = 40;

/// How far inside the right edge of a window's visible bounds the pointer
/// first tries to grab that edge, and how far outside it looks for it, in
/// the band the system resizes it by, in pixels at 96 DPI.
const EDGE_INSIDE: i32 = 4;
const EDGE_OUTSIDE: i32 = 16;

/// Where the pointer holds `window`, whose visible bounds are `rect`, by
/// its title bar: right of its middle, where no tab or button of a notepad
/// is, and as far down from its top as the window's DPI scales
/// [`FIRST_TRY`], or further down, at the first point the window says is
/// its caption. Fails the test when it says so nowhere near its top.
pub fn title_bar(window: WindowId, rect: Rect) -> (i32, i32) {
    let hwnd = window.raw() as isize;
    let x = rect.x + rect.width * 2 / 3;
    let scaled = scale(hwnd);
    (scaled(FIRST_TRY)..scaled(DEEPEST))
        .map(|down| (x, rect.y + down))
        .find(|&at| answers(hwnd, at, win32::HTCAPTION))
        .expect("the window says its caption is nowhere near its top")
}

/// Where the pointer holds `window`, whose visible bounds are `rect`, by
/// its right edge: halfway down, at the first point from a little inside
/// that edge outwards that the window says is its right edge. Fails the
/// test when it says so nowhere near it.
pub fn right_edge(window: WindowId, rect: Rect) -> (i32, i32) {
    let hwnd = window.raw() as isize;
    let right = rect.x + rect.width;
    let scaled = scale(hwnd);
    (right - scaled(EDGE_INSIDE)..right + scaled(EDGE_OUTSIDE))
        .map(|x| (x, rect.center_y()))
        .find(|&at| answers(hwnd, at, win32::HTRIGHT))
        .expect("the window says its right edge is nowhere near it")
}

/// Pixels at 96 DPI as many as they are at the DPI of `hwnd`.
fn scale(hwnd: isize) -> impl Fn(i32) -> i32 {
    // SAFETY: a read-only query, which gives 0 for a window that is gone.
    let dpi = match unsafe { win32::GetDpiForWindow(hwnd) } {
        0 => 96,
        dpi => dpi as i32,
    };
    move |pixels| pixels * dpi / 96
}

/// Whether `hwnd` says `at`, in the display's pixels, is the part of it
/// that `WM_NCHITTEST` names `code`.
fn answers(hwnd: isize, (x, y): (i32, i32), code: usize) -> bool {
    let mut point = win32::Point { x, y };
    let mut answer = 0usize;
    // SAFETY: the conversion reads the window's DPI and writes the point
    // on the stack, which it leaves as it is for a window that is aware of
    // the display's DPI. WM_NCHITTEST only asks the window what is at the
    // point, and its answer is written on the stack, within the timeout
    // whatever the window does.
    let sent = unsafe {
        win32::PhysicalToLogicalPointForPerMonitorDPI(hwnd, &mut point);
        let packed = (u32::from(point.y as u16) << 16) | u32::from(point.x as u16);
        let (hit, flags) = (win32::WM_NCHITTEST, win32::SMTO_ABORTIFHUNG);
        win32::SendMessageTimeoutW(hwnd, hit, 0, packed as isize, flags, 100, &mut answer)
    };
    sent != 0 && answer == code
}
