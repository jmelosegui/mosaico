use std::mem;

use mosaico_core::{Rect, WindowResult};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTOPRIMARY, MONITORINFO, MonitorFromWindow,
};

/// Returns the work area of the primary monitor.
///
/// The work area excludes the taskbar and any docked toolbars.
/// This is where windows should be tiled.
pub fn primary_work_area() -> WindowResult<Rect> {
    // MonitorFromWindow with a null HWND and MONITOR_DEFAULTTOPRIMARY
    // returns the primary monitor handle.
    let monitor = unsafe { MonitorFromWindow(HWND::default(), MONITOR_DEFAULTTOPRIMARY) };
    work_area_for_monitor(monitor)
}

/// Returns the work area of the monitor containing the given window.
pub fn work_area_for_window(hwnd: HWND) -> WindowResult<Rect> {
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY) };
    work_area_for_monitor(monitor)
}

/// Queries the work area for a given monitor handle.
fn work_area_for_monitor(monitor: windows::Win32::Graphics::Gdi::HMONITOR) -> WindowResult<Rect> {
    let mut info = MONITORINFO {
        cbSize: mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    // SAFETY: GetMonitorInfoW fills the MONITORINFO struct with
    // monitor dimensions. We set cbSize as required by the API.
    let success = unsafe { GetMonitorInfoW(monitor, &mut info) };

    if !success.as_bool() {
        return Err("Failed to get monitor info".into());
    }

    let rc = info.rcWork;
    Ok(Rect::new(
        rc.left,
        rc.top,
        rc.right - rc.left,
        rc.bottom - rc.top,
    ))
}
