use mosaico_core::{Rect, WindowResult};

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, RealGetWindowClassW, SWP_FRAMECHANGED,
    SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOSENDCHANGING, SWP_NOZORDER, SetWindowPos,
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

    /// Returns whether this looks like a real application window.
    ///
    /// Checks for a caption bar (`WS_CAPTION`) and rejects tool windows
    /// (`WS_EX_TOOLWINDOW`). This filters out internal helper windows,
    /// tooltips, floating toolbars, and other non-application surfaces
    /// that should never be tiled.
    pub fn is_app_window(&self) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::{
            GWL_EXSTYLE, GWL_STYLE, GetWindowLongPtrW, WS_CAPTION, WS_EX_TOOLWINDOW,
        };

        unsafe {
            let style = GetWindowLongPtrW(self.hwnd, GWL_STYLE) as u32;
            let ex_style = GetWindowLongPtrW(self.hwnd, GWL_EXSTYLE) as u32;

            let has_caption = (style & WS_CAPTION.0) == WS_CAPTION.0;
            let is_tool = (ex_style & WS_EX_TOOLWINDOW.0) == WS_EX_TOOLWINDOW.0;

            has_caption && !is_tool
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

        let x = rect.x - border.left;
        let y = rect.y - border.top;
        let cx = rect.width + border.left + border.right;
        let cy = rect.height + border.top + border.bottom;

        // SWP_NOSENDCHANGING — suppresses WM_WINDOWPOSCHANGING so the
        //   window cannot reject or modify the requested dimensions.
        // SWP_FRAMECHANGED — forces WM_NCCALCSIZE so GPU-composited
        //   apps (Chrome, Electron) recalculate their client area and
        //   update the rendering surface.
        // SWP_NOCOPYBITS — discards old client-area contents instead
        //   of blitting them, avoiding stale surfaces.
        let flags =
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_FRAMECHANGED | SWP_NOCOPYBITS;

        // SAFETY: SetWindowPos with a valid HWND is safe.
        unsafe { SetWindowPos(self.hwnd, None, x, y, cx, cy, flags)? };
        Ok(())
    }

    fn invalidate(&self) {
        use windows::Win32::Graphics::Gdi::{
            RDW_ALLCHILDREN, RDW_ERASE, RDW_FRAME, RDW_INVALIDATE, RDW_UPDATENOW, RedrawWindow,
        };

        // SAFETY: RedrawWindow forces a complete repaint of the window,
        // its frame, and all child windows. The aggressive flag
        // combination ensures GPU-composited apps (Chrome, Electron)
        // refresh their rendering surface after a programmatic resize.
        unsafe {
            let _ = RedrawWindow(
                Some(self.hwnd),
                None,
                None,
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_FRAME | RDW_ERASE | RDW_ALLCHILDREN,
            );
        }
    }

    fn is_visible(&self) -> bool {
        // SAFETY: IsWindowVisible is a simple query that returns a BOOL.
        unsafe { IsWindowVisible(self.hwnd).as_bool() }
    }
}
