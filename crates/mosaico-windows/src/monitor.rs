use std::mem;

use mosaico_core::{Rect, WindowResult};
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITOR_DEFAULTTOPRIMARY, MONITORINFO,
    MonitorFromWindow,
};
use windows::core::BOOL;

/// Information about a connected monitor.
pub struct MonitorInfo {
    /// Unique identifier (HMONITOR cast to usize).
    pub id: usize,
    /// Work area excluding the taskbar and docked toolbars.
    pub work_area: Rect,
}

/// Returns the monitor ID for the monitor containing the given window.
///
/// Falls back to the primary monitor if the window handle is invalid.
pub fn monitor_id_for_window(hwnd: usize) -> usize {
    let hmonitor = unsafe { MonitorFromWindow(HWND(hwnd as *mut _), MONITOR_DEFAULTTOPRIMARY) };
    hmonitor.0 as usize
}

/// Enumerates all connected monitors, sorted left-to-right.
pub fn enumerate_monitors() -> WindowResult<Vec<MonitorInfo>> {
    let mut monitors: Vec<MonitorInfo> = Vec::new();

    // SAFETY: EnumDisplayMonitors calls our callback for each monitor.
    // We pass a pointer to our Vec as LPARAM. The callback runs
    // synchronously, so the Vec outlives the call.
    let success = unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_callback),
            LPARAM(&mut monitors as *mut _ as isize),
        )
    };

    if !success.as_bool() {
        return Err("Failed to enumerate monitors".into());
    }

    // Sort left-to-right so "next" and "previous" are spatial.
    monitors.sort_by_key(|m| m.work_area.x);

    Ok(monitors)
}

/// Callback invoked by `EnumDisplayMonitors` for each connected monitor.
unsafe extern "system" fn monitor_enum_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _lprect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    // SAFETY: lparam is a pointer to our Vec<MonitorInfo>.
    let monitors = unsafe { &mut *(lparam.0 as *mut Vec<MonitorInfo>) };

    if let Ok(work_area) = work_area_for_monitor(hmonitor) {
        monitors.push(MonitorInfo {
            id: hmonitor.0 as usize,
            work_area,
        });
    }

    BOOL(1) // Continue enumeration
}

/// Queries the work area for a given monitor handle.
fn work_area_for_monitor(monitor: HMONITOR) -> WindowResult<Rect> {
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
