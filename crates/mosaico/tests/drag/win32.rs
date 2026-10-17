//! The Win32 calls the drag and drop tests make themselves: sending mouse
//! input, reading the pointer, and asking a window what is under a point.

#[repr(C)]
pub struct MouseInput {
    pub dx: i32,
    pub dy: i32,
    pub mouse_data: u32,
    pub flags: u32,
    pub time: u32,
    pub extra_info: usize,
}

/// An `INPUT` of the mouse kind.
#[repr(C)]
pub struct Input {
    pub kind: u32,
    pub mi: MouseInput,
}

/// A `POINT`.
#[repr(C)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub const INPUT_MOUSE: u32 = 0;
pub const MOUSEEVENTF_MOVE: u32 = 0x0001;
pub const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
pub const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
pub const MOUSEEVENTF_VIRTUALDESK: u32 = 0x4000;
pub const MOUSEEVENTF_ABSOLUTE: u32 = 0x8000;
pub const SM_XVIRTUALSCREEN: i32 = 76;
pub const SM_YVIRTUALSCREEN: i32 = 77;
pub const SM_CXVIRTUALSCREEN: i32 = 78;
pub const SM_CYVIRTUALSCREEN: i32 = 79;
pub const WM_NCHITTEST: u32 = 0x0084;
pub const HTCAPTION: usize = 2;
pub const HTRIGHT: usize = 11;
pub const SMTO_ABORTIFHUNG: u32 = 0x0002;

#[link(name = "user32")]
unsafe extern "system" {
    pub fn SendInput(count: u32, inputs: *const Input, size: i32) -> u32;
    pub fn GetSystemMetrics(index: i32) -> i32;
    pub fn GetCursorPos(point: *mut Point) -> i32;
    pub fn GetDpiForWindow(hwnd: isize) -> u32;
    pub fn PhysicalToLogicalPointForPerMonitorDPI(hwnd: isize, point: *mut Point) -> i32;
    pub fn SendMessageTimeoutW(
        hwnd: isize,
        msg: u32,
        wparam: usize,
        lparam: isize,
        flags: u32,
        timeout: u32,
        result: *mut usize,
    ) -> isize;
}
