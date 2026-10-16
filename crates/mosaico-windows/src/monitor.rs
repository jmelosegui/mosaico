use std::mem;

use mosaico_core::{Error, MonitorId, MonitorKey, Rect, Result, WindowId};
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITOR_DEFAULTTOPRIMARY, MONITORINFO,
    MONITORINFOEXW, MonitorFromWindow,
};
use windows::core::BOOL;

use crate::monitor_identity;

/// `MONITORINFOF_PRIMARY` flag in `MONITORINFO::dwFlags`.
const MONITORINFOF_PRIMARY: u32 = 1;

/// Information about a connected monitor.
pub struct MonitorInfo {
    /// Runtime handle: the HMONITOR value.
    ///
    /// Only valid until the next display change: Windows hands out new
    /// handles whenever the configuration changes. Use `key` to
    /// recognise a monitor across changes.
    pub id: MonitorId,
    /// Identity that survives reconnects. See [`MonitorKey`].
    pub key: MonitorKey,
    /// Work area excluding the taskbar and docked toolbars.
    pub work_area: Rect,
    /// Whether this is the primary monitor.
    pub primary: bool,
}

impl mosaico_core::Monitor for MonitorInfo {
    fn key(&self) -> &MonitorKey {
        &self.key
    }

    fn work_area(&self) -> Rect {
        self.work_area
    }

    fn is_primary(&self) -> bool {
        self.primary
    }
}

/// Returns the monitor ID for the monitor containing the given window.
///
/// Falls back to the primary monitor if the window handle is invalid.
pub fn monitor_id_for_window(window: WindowId) -> MonitorId {
    let hwnd = crate::window::Window::from_id(window).hwnd();
    // SAFETY: MonitorFromWindow is a read-only query on a window handle.
    let hmonitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY) };
    MonitorId::from_raw(hmonitor.0 as usize as u64)
}

/// A monitor as seen by `EnumDisplayMonitors`, before its key is known.
struct RawMonitor {
    id: MonitorId,
    work_area: Rect,
    primary: bool,
    gdi_name: String,
}

/// Enumerates all connected monitors, sorted left-to-right.
pub fn enumerate_monitors() -> Result<Vec<MonitorInfo>> {
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
    let mut monitors: Vec<MonitorInfo> = raw
        .into_iter()
        .map(|m| MonitorInfo {
            id: m.id,
            key: identities.remove(&m.gdi_name).unwrap_or_default(),
            work_area: m.work_area,
            primary: m.primary,
        })
        .collect();

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

/// Queries the work area, primary flag and GDI device name of a monitor.
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
        id: MonitorId::from_raw(monitor.0 as usize as u64),
        work_area: rect_from_win32(&info.monitorInfo.rcWork),
        primary: info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0,
        gdi_name: String::from_utf16_lossy(&info.szDevice[..name_len]),
    })
}
