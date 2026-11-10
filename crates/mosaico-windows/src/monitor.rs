use std::mem;

use mosaico_core::tiler::Display;
use mosaico_core::{Error, MonitorId, Rect, Result, WindowId};
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITOR_DEFAULTTOPRIMARY, MONITORINFO,
    MONITORINFOEXW, MonitorFromWindow,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::core::BOOL;

use crate::monitor_identity;

/// `MONITORINFOF_PRIMARY` flag in `MONITORINFO::dwFlags`.
const MONITORINFOF_PRIMARY: u32 = 1;

/// Returns the monitor ID for the monitor containing the given window.
///
/// `MonitorFromWindow` picks the monitor with the largest share of the
/// window, uses the restored position of a minimized window, and falls
/// back to the primary monitor for a window on no monitor. For a handle
/// that is no longer valid it returns a null monitor handle, which comes
/// out as an id that is in no monitor list.
pub(crate) fn monitor_id_for_window(window: WindowId) -> MonitorId {
    let hwnd = crate::window::Window::from_id(window).hwnd();
    // SAFETY: MonitorFromWindow is a read-only query on a window handle.
    let hmonitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY) };
    monitor_id(hmonitor)
}

/// The core id of a monitor, which is its `HMONITOR` value.
///
/// Only valid until the next display change: Windows hands out new
/// handles whenever the configuration changes, even for monitors that
/// stayed connected. The monitor's key recognises it across changes.
fn monitor_id(monitor: HMONITOR) -> MonitorId {
    MonitorId::from_raw(monitor.0 as usize as u64)
}

/// A monitor as seen by `EnumDisplayMonitors`, before its key is known.
struct RawMonitor {
    id: MonitorId,
    bounds: Rect,
    work_area: Rect,
    primary: bool,
    gdi_name: String,
    scale: f64,
}

/// Enumerates all connected monitors, in the order `EnumDisplayMonitors`
/// reports them. [`Enumerator::monitors`] puts them left to right.
///
/// [`Enumerator::monitors`]: mosaico_core::tiler::Enumerator::monitors
pub(crate) fn enumerate_monitors() -> Result<Vec<Display>> {
    let mut raw: Vec<RawMonitor> = Vec::new();

    // SAFETY: EnumDisplayMonitors calls our callback for each monitor.
    // We pass a pointer to our Vec as LPARAM. The callback runs
    // synchronously, so the Vec outlives the call.
    let success = unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_callback),
            LPARAM(&mut raw as *mut _ as isize),
        )
    };

    if !success.as_bool() {
        return Err(Error::platform(
            "list monitors",
            "EnumDisplayMonitors failed",
        ));
    }

    let mut identities = monitor_identity::display_identities();
    Ok(raw
        .into_iter()
        .map(|m| {
            let identity = identities.remove(&m.gdi_name);
            let (key, name) = identity.map_or_else(Default::default, |i| (i.key, i.name));
            Display {
                id: m.id,
                key,
                bounds: m.bounds,
                work_area: m.work_area,
                primary: m.primary,
                name,
                device: m.gdi_name,
                scale: m.scale,
            }
        })
        .collect())
}

/// Callback invoked by `EnumDisplayMonitors` for each connected monitor.
unsafe extern "system" fn monitor_enum_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _lprect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    // SAFETY: lparam is a pointer to our Vec<RawMonitor>.
    let monitors = unsafe { &mut *(lparam.0 as *mut Vec<RawMonitor>) };

    if let Ok(monitor) = query_monitor(hmonitor) {
        monitors.push(monitor);
    }

    BOOL(1) // Continue enumeration
}

/// Converts a Win32 `RECT` (left/top/right/bottom) to a
/// mosaico `Rect` (x/y/width/height).
pub(crate) fn rect_from_win32(r: &RECT) -> Rect {
    Rect::new(r.left, r.top, r.right - r.left, r.bottom - r.top)
}

/// Queries the bounds, work area, primary flag, GDI device name and scale
/// of a monitor.
fn query_monitor(monitor: HMONITOR) -> Result<RawMonitor> {
    let mut info = MONITORINFOEXW {
        monitorInfo: MONITORINFO {
            cbSize: mem::size_of::<MONITORINFOEXW>() as u32,
            ..Default::default()
        },
        ..Default::default()
    };

    // SAFETY: GetMonitorInfoW fills the struct. cbSize tells it the
    // buffer is the extended version, which also carries szDevice.
    let success = unsafe { GetMonitorInfoW(monitor, &mut info.monitorInfo) };

    if !success.as_bool() {
        return Err(Error::platform(
            "read the monitor info",
            "GetMonitorInfoW failed",
        ));
    }

    let name_len = info
        .szDevice
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(info.szDevice.len());

    Ok(RawMonitor {
        id: monitor_id(monitor),
        bounds: rect_from_win32(&info.monitorInfo.rcMonitor),
        work_area: rect_from_win32(&info.monitorInfo.rcWork),
        primary: info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0,
        gdi_name: String::from_utf16_lossy(&info.szDevice[..name_len]),
        scale: scale_from_dpi(effective_dpi(monitor)),
    })
}

/// The effective DPI of a monitor, None when it cannot be read.
fn effective_dpi(monitor: HMONITOR) -> Option<u32> {
    let (mut x, mut y) = (0, 0);
    // SAFETY: GetDpiForMonitor writes two integers on the stack.
    unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y).ok()? };
    Some(x)
}

/// The scale factor for a DPI, 1.0 at the standard 96 and 1.0 when the
/// DPI is unknown.
fn scale_from_dpi(dpi: Option<u32>) -> f64 {
    match dpi {
        Some(dpi) if dpi > 0 => f64::from(dpi) / 96.0,
        _ => 1.0,
    }
}

#[cfg(test)]
#[path = "monitor_tests.rs"]
mod tests;
