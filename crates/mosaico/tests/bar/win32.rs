//! The Win32 calls the bar click tests make themselves, besides those of
//! the pointer: finding the bar's window, its monitor, its style and its
//! process, having the bar set the pointer's shape and reading it, and how
//! the system routes the wheel.

use super::win32::Point;

/// A `RECT`.
#[repr(C)]
#[derive(Default)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// A `CURSORINFO`, with the `POINT` of the pointer's position as `x` and
/// `y`.
#[repr(C)]
#[derive(Default)]
pub struct CursorInfo {
    pub size: u32,
    pub flags: u32,
    pub cursor: isize,
    pub x: i32,
    pub y: i32,
}

/// A `WNDENUMPROC`.
pub type EnumProc = unsafe extern "system" fn(hwnd: isize, lparam: isize) -> i32;

/// `IDC_HAND`, the hand the bar shows over what takes clicks.
pub const IDC_HAND: usize = 32649;
pub const MONITOR_DEFAULTTONULL: u32 = 0;
pub const MONITOR_DEFAULTTOPRIMARY: u32 = 1;
pub const GWL_EXSTYLE: i32 = -20;
/// The style of a window that lets the pointer through to the windows
/// under it.
pub const WS_EX_TRANSPARENT: i32 = 0x0020;
pub const WM_SETCURSOR: u32 = 0x0020;
/// The `lparam` of the `WM_SETCURSOR` the system sends the window under
/// the pointer as the pointer moves over its client area: `HTCLIENT` in
/// the low word, and `WM_MOUSEMOVE` in the high one.
pub const MOVED_OVER_CLIENT: isize = (0x0200 << 16) | 1;
pub const SPI_GETMOUSEWHEELROUTING: u32 = 0x201C;
/// The wheel goes to the window under the pointer, as with "Scroll
/// inactive windows when I hover over them" on.
pub const MOUSEWHEEL_ROUTING_MOUSE_POS: u32 = 2;

#[link(name = "user32")]
unsafe extern "system" {
    pub fn EnumWindows(each: EnumProc, lparam: isize) -> i32;
    pub fn GetClassNameW(hwnd: isize, name: *mut u16, max: i32) -> i32;
    pub fn IsWindowVisible(hwnd: isize) -> i32;
    pub fn GetWindowRect(hwnd: isize, rect: *mut Rect) -> i32;
    pub fn GetWindowLongW(hwnd: isize, index: i32) -> i32;
    pub fn GetWindowThreadProcessId(hwnd: isize, process: *mut u32) -> u32;
    pub fn MonitorFromWindow(hwnd: isize, flags: u32) -> isize;
    pub fn MonitorFromPoint(point: Point, flags: u32) -> isize;
    pub fn WindowFromPoint(point: Point) -> isize;
    pub fn GetCursorInfo(info: *mut CursorInfo) -> i32;
    pub fn LoadCursorW(instance: isize, name: usize) -> isize;
    pub fn SystemParametersInfoW(action: u32, param: u32, value: *mut u32, ini: u32) -> i32;
}
