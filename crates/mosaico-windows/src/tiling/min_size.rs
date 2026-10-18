//! The least size an application says its window can have, read for the
//! tiler's `Desktop::min_size`.
//!
//! The window is asked with `WM_GETMINMAXINFO` through
//! `SendMessageTimeoutW`, as the system asks it when the user sizes the
//! window, and what it answers in `ptMinTrackSize` is its least size
//! with the invisible frame the shadow takes. The window gets
//! [`QUERY_TIMEOUT_MS`] to answer, so one that is not responding is read
//! as having no minimum rather than held up for. The structure is
//! filled with the system's least size first: a window that does not
//! handle the message, or leaves the field alone, answers that, and
//! [`MinSize::within`] reads it as no minimum.

use mosaico_core::MinSize;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi};
use windows::Win32::UI::WindowsAndMessaging::{
    MINMAXINFO, SM_CXMINTRACK, SM_CYMINTRACK, SMTO_ABORTIFHUNG, SendMessageTimeoutW,
    WM_GETMINMAXINFO,
};

use crate::frame;
use crate::window::Window;

/// How long a window has to answer. One that responds takes microseconds.
const QUERY_TIMEOUT_MS: u32 = 50;

/// The minimum visible size of `window`, or `None` when it has none or
/// does not answer in time.
pub(super) fn read(window: &Window) -> Option<MinSize> {
    let hwnd = window.hwnd();
    let floor = system_floor(hwnd);
    let mut info = MINMAXINFO::default();
    info.ptMinTrackSize.x = floor.width;
    info.ptMinTrackSize.y = floor.height;
    let mut result = 0usize;
    // SAFETY: WM_GETMINMAXINFO carries a pointer to a MINMAXINFO that the
    // window fills in; `info` outlives the call, which returns within the
    // timeout whatever the window does.
    let sent = unsafe {
        SendMessageTimeoutW(
            hwnd,
            WM_GETMINMAXINFO,
            WPARAM(0),
            LPARAM(&mut info as *mut MINMAXINFO as isize),
            SMTO_ABORTIFHUNG,
            QUERY_TIMEOUT_MS,
            Some(&mut result),
        )
    };
    if sent.0 == 0 {
        return None;
    }
    let track = info.ptMinTrackSize;
    let outer = MinSize::new(track.x, track.y);
    let margin = frame::border_offset(hwnd).ok()?;
    let found = MinSize::within(
        outer,
        floor,
        MinSize::new(margin.left + margin.right, margin.top + margin.bottom),
    );
    (!found.is_none()).then_some(found)
}

/// The least size the system gives any window, at the DPI of `hwnd`.
fn system_floor(hwnd: HWND) -> MinSize {
    // SAFETY: both are read-only queries.
    unsafe {
        let dpi = GetDpiForWindow(hwnd).max(96);
        MinSize::new(
            GetSystemMetricsForDpi(SM_CXMINTRACK, dpi),
            GetSystemMetricsForDpi(SM_CYMINTRACK, dpi),
        )
    }
}
