use mosaico_core::{Rect, WindowResult};

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowRect, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, RealGetWindowClassW,
};

/// A window on the Windows platform, wrapping a Win32 `HWND`.
///
/// `HWND` is an opaque handle — a number that identifies a window to the OS.
/// This struct holds that handle and queries the OS lazily for metadata.
#[derive(Debug, Clone, Copy)]
pub struct Window {
    hwnd: HWND,
}

impl Window {
    /// Creates a new `Window` from a raw `HWND`.
    pub fn new(hwnd: HWND) -> Self {
        Self { hwnd }
    }

    /// Returns the raw window handle.
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }
}

impl mosaico_core::Window for Window {
    fn title(&self) -> WindowResult<String> {
        // SAFETY: GetWindowTextLengthW and GetWindowTextW are safe to call
        // with a valid HWND. They read window text without modifying state.
        unsafe {
            let length = GetWindowTextLengthW(self.hwnd);
            if length == 0 {
                return Ok(String::new());
            }

            // +1 for the null terminator that Windows requires
            let mut buffer = vec![0u16; (length + 1) as usize];
            let copied = GetWindowTextW(self.hwnd, &mut buffer);
            Ok(String::from_utf16_lossy(&buffer[..copied as usize]))
        }
    }

    fn class(&self) -> WindowResult<String> {
        // SAFETY: RealGetWindowClassW reads the window class name.
        // 256 is the maximum class name length in Win32.
        unsafe {
            let mut buffer = [0u16; 256];
            let length = RealGetWindowClassW(self.hwnd, &mut buffer);
            Ok(String::from_utf16_lossy(&buffer[..length as usize]))
        }
    }

    fn rect(&self) -> WindowResult<Rect> {
        let mut rect = windows::Win32::Foundation::RECT::default();

        // SAFETY: GetWindowRect writes the window's bounding rectangle
        // into the provided RECT pointer. We pass a valid mutable reference.
        unsafe {
            GetWindowRect(self.hwnd, &mut rect)?;
        }

        Ok(Rect::new(
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
        ))
    }

    fn is_visible(&self) -> bool {
        // SAFETY: IsWindowVisible is a simple query that returns a BOOL.
        unsafe { IsWindowVisible(self.hwnd).as_bool() }
    }
}
