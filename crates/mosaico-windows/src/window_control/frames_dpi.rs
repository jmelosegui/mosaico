//! Where a window that does not follow the display's scale can be put.
//!
//! The daemon is per-monitor DPI aware: it reads and moves windows in the
//! display's own pixels. A window that is not has pixels of its own, at
//! 96 DPI when it is not DPI aware at all, and on a display scaled up
//! each of them covers more than one of the display's. Windows maps every
//! move made to such a window to its pixels and back, rounding the
//! position and the size each on their own, halves away from zero, the
//! way `MulDiv` does. A frame less than one of the window's pixels from
//! where it is would leave it there, and the window would look as if it
//! had not handled the frame yet. So a frame for such a window is mapped
//! the same way before it is compared or posted.
//!
//! `PhysicalToLogicalPointForPerMonitorDPI` maps points, but only points
//! within the window, which the corners of a frame mostly are not.
//!
//! The mapping has been checked on displays at 125% and 200%, 120 and
//! 192 DPI: 150 and 175%, a monitor whose origin is not a multiple of its
//! scale, and monitors at different scales have not been tried. A miss
//! holds the next frame back for one limit at most.

use mosaico_core::Rect;
use windows::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONEAREST, MonitorFromWindow};
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_PER_MONITOR_AWARE, GetAwarenessFromDpiAwarenessContext, GetDpiForMonitor,
    GetDpiForWindow, GetSystemMetricsForDpi, GetThreadDpiAwarenessContext,
    GetWindowDpiAwarenessContext, MDT_EFFECTIVE_DPI,
};

use windows::Win32::UI::WindowsAndMessaging::{SM_CXPADDEDBORDER, SM_CXSIZEFRAME, SM_CYSIZEFRAME};

use crate::frame::BorderOffset;
use crate::window::Window;

/// Whether Windows maps the moves the calling thread makes to the window
/// to pixels of the window's own: the thread is per-monitor DPI aware, and
/// the window is not. A thread that is not aware reads and moves windows
/// in scaled units already.
pub(super) fn scaled(win: Window) -> bool {
    let aware = DPI_AWARENESS_PER_MONITOR_AWARE;
    // SAFETY: read-only queries of the DPI awareness of the calling
    // thread and of a window. A window that is gone has none, which is
    // not per-monitor awareness.
    unsafe {
        GetAwarenessFromDpiAwarenessContext(GetThreadDpiAwarenessContext()) == aware
            && GetAwarenessFromDpiAwarenessContext(GetWindowDpiAwarenessContext(win.hwnd()))
                != aware
    }
}

/// Where the window ends up when moved to the outer bounds `rect`, once
/// Windows has mapped the move to the window's own pixels and back. The
/// rect as it is when either DPI cannot be read, once the window is gone.
pub(super) fn reachable(win: Window, rect: Rect) -> Rect {
    let (Some(own), Some(display)) = (window_dpi(win), display_dpi(win)) else {
        return rect;
    };
    let fit = |value: i32| mul_div(mul_div(value, own, display), display, own);
    Rect::new(fit(rect.x), fit(rect.y), fit(rect.width), fit(rect.height))
}

/// The DPI of the window's own pixels: 96 for a window that is not DPI
/// aware, the system's for one that is system aware.
fn window_dpi(win: Window) -> Option<i32> {
    // SAFETY: a read-only query, which gives 0 for a window that is gone.
    let dpi = unsafe { GetDpiForWindow(win.hwnd()) };
    i32::try_from(dpi).ok().filter(|&dpi| dpi > 0)
}

/// The DPI of the display the window is on: for a minimized window, the
/// one it is restored to.
pub(super) fn display_dpi(win: Window) -> Option<i32> {
    let (mut x, mut y) = (0, 0);
    // SAFETY: MonitorFromWindow is a read-only query, and GetDpiForMonitor
    // writes two integers on the stack.
    unsafe {
        let monitor = MonitorFromWindow(win.hwnd(), MONITOR_DEFAULTTONEAREST);
        GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y).ok()?;
    }
    i32::try_from(x).ok().filter(|&dpi| dpi > 0)
}

/// How wide the frame Windows draws around a window is at a DPI, its
/// invisible borders in it: on the left and right, and at the bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FrameWidths {
    pub(super) side: i32,
    pub(super) bottom: i32,
}

/// The frame widths at `dpi`: `SM_CXSIZEFRAME` and `SM_CYSIZEFRAME`, each
/// with `SM_CXPADDEDBORDER`. `None` when they cannot be read.
pub(super) fn frame_widths(dpi: i32) -> Option<FrameWidths> {
    let dpi = u32::try_from(dpi).ok()?;
    let metric = |index| {
        // SAFETY: GetSystemMetricsForDpi is a read-only query, which gives
        // 0 for what it cannot answer.
        unsafe { GetSystemMetricsForDpi(index, dpi) }
    };
    let (cx, cy, padded) = (
        metric(SM_CXSIZEFRAME),
        metric(SM_CYSIZEFRAME),
        metric(SM_CXPADDEDBORDER),
    );
    (cx > 0 && cy > 0).then_some(FrameWidths {
        side: cx + padded,
        bottom: cy + padded,
    })
}

/// Invisible borders read where the frame was `then` wide, as they are
/// where it is `now` wide: each side moved by the difference. The top,
/// which has none on Windows 10 and 11, is left alone.
pub(super) fn shift_border(
    border: &BorderOffset,
    then: FrameWidths,
    now: FrameWidths,
) -> BorderOffset {
    let side = now.side - then.side;
    BorderOffset {
        left: border.left + side,
        top: border.top,
        right: border.right + side,
        bottom: border.bottom + now.bottom - then.bottom,
    }
}

/// Invisible borders drawn at the DPI `from`, as they are at `to`.
pub(super) fn scale_border(border: &BorderOffset, to: i32, from: i32) -> BorderOffset {
    let scale = |value: i32| mul_div(value, to, from);
    BorderOffset {
        left: scale(border.left),
        top: scale(border.top),
        right: scale(border.right),
        bottom: scale(border.bottom),
    }
}

/// `value * num / den`, rounded to the nearest integer and halves away
/// from zero, as `MulDiv` rounds. `den` is positive.
fn mul_div(value: i32, num: i32, den: i32) -> i32 {
    let product = i64::from(value) * i64::from(num);
    let den = i64::from(den);
    let rounded = (2 * product.abs() + den) / (2 * den) * product.signum();
    i32::try_from(rounded).unwrap_or(value)
}

#[cfg(test)]
#[path = "frames_dpi_tests.rs"]
mod tests;
