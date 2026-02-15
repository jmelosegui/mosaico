use mosaico_core::{Rect, WindowResult};

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, RealGetWindowClassW, SWP_NOACTIVATE,
    SWP_NOZORDER, SetWindowPos,
};

use crate::frame;

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

    /// Creates a new `Window` from a raw handle value (pointer-sized integer).
    ///
    /// This allows callers to construct a `Window` without depending on the
    /// `windows` crate directly.
    pub fn from_raw(handle: usize) -> Self {
        Self {
            hwnd: HWND(handle as *mut _),
        }
    }

    /// Returns the raw window handle.
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Sets this window as the foreground (focused) window.
    pub fn set_foreground(&self) {
        use windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow;

        // SAFETY: SetForegroundWindow is safe to call with a valid HWND.
        unsafe {
            let _ = SetForegroundWindow(self.hwnd);
        }
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
        let frame = frame::visible_rect(self.hwnd)?;

        Ok(Rect::new(
            frame.left,
            frame.top,
            frame.right - frame.left,
            frame.bottom - frame.top,
        ))
    }

    fn set_rect(&self, rect: &mosaico_core::Rect) -> WindowResult<()> {
        // Compensate for invisible borders so the visible portion
        // lands exactly at the requested position and size.
        let border = frame::border_offset(self.hwnd)?;

        // SAFETY: SetWindowPos moves and resizes the window.
        // SWP_NOZORDER — don't change the Z-order (stacking position).
        // SWP_NOACTIVATE — don't steal focus from the current window.
        unsafe {
            SetWindowPos(
                self.hwnd,
                None,
                rect.x - border.left,
                rect.y - border.top,
                rect.width + border.left + border.right,
                rect.height + border.top + border.bottom,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
        }
        Ok(())
    }

    fn is_visible(&self) -> bool {
        // SAFETY: IsWindowVisible is a simple query that returns a BOOL.
        unsafe { IsWindowVisible(self.hwnd).as_bool() }
    }
}
