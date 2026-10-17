//! Where the pointer grabs a window by its title bar, for the drag and
//! drop tests: a point the window itself says is its caption, as it
//! answers `WM_NCHITTEST`, below the band at its top that resizes it.

use mosaico_core::{Rect, WindowId};

use super::win32;

/// How far below the top of a window's visible bounds the pointer first
/// tries to grab its caption, and how far down it looks for it, in pixels
/// at 96 DPI.
const FIRST_TRY: i32 = 12;
const DEEPEST: i32 = 40;

/// Where the pointer holds `window`, whose visible bounds are `rect`, by
/// its title bar: right of its middle, where no tab or button of a notepad
/// is, and as far down from its top as the window's DPI scales
/// [`FIRST_TRY`], or further down, at the first point the window says is
/// its caption. Fails the test when it says so nowhere near its top.
pub fn title_bar(window: WindowId, rect: Rect) -> (i32, i32) {
    let hwnd = window.raw() as isize;
    // SAFETY: a read-only query, which gives 0 for a window that is gone.
    let dpi = match unsafe { win32::GetDpiForWindow(hwnd) } {
        0 => 96,
        dpi => dpi as i32,
    };
    let x = rect.x + rect.width * 2 / 3;
    let scaled = |pixels: i32| pixels * dpi / 96;
    (scaled(FIRST_TRY)..scaled(DEEPEST))
        .map(|down| (x, rect.y + down))
        .find(|&at| is_caption(hwnd, at))
        .expect("the window says its caption is nowhere near its top")
}

/// Whether `hwnd` says `at`, in the display's pixels, is its caption.
fn is_caption(hwnd: isize, (x, y): (i32, i32)) -> bool {
    let mut point = win32::Point { x, y };
    let mut code = 0usize;
    // SAFETY: the conversion reads the window's DPI and writes the point
    // on the stack, which it leaves as it is for a window that is aware of
    // the display's DPI. WM_NCHITTEST only asks the window what is at the
    // point, and its answer is written on the stack, within the timeout
    // whatever the window does.
    let sent = unsafe {
        win32::PhysicalToLogicalPointForPerMonitorDPI(hwnd, &mut point);
        let packed = (u32::from(point.y as u16) << 16) | u32::from(point.x as u16);
        let (hit, flags) = (win32::WM_NCHITTEST, win32::SMTO_ABORTIFHUNG);
        win32::SendMessageTimeoutW(hwnd, hit, 0, packed as isize, flags, 100, &mut code)
    };
    sent != 0 && code == win32::HTCAPTION
}
