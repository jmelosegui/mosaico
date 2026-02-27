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

    /// Returns whether this window needs `SWP_FRAMECHANGED` to update
    /// its rendering surface after a programmatic resize.
    ///
    /// Chromium-based apps (Chrome, Edge, Electron) use a GPU compositor
    /// that only repaints when `WM_NCCALCSIZE` fires. Without
    /// `SWP_FRAMECHANGED`, they render a blank/stale surface.
    fn needs_frame_changed(&self) -> bool {
        let class = mosaico_core::Window::class(self).unwrap_or_default();
        class == "Chrome_WidgetWin_1" || class == "MozillaWindowClass"
    }

    /// Hides the window without destroying it.
    pub fn hide(&self) {
        use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, ShowWindow};
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    /// Shows a previously hidden window.
    pub fn show(&self) {
        use windows::Win32::UI::WindowsAndMessaging::{SW_SHOWNOACTIVATE, ShowWindow};
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
        }
    }

    /// Minimizes the window.
    pub fn minimize(&self) {
        use windows::Win32::UI::WindowsAndMessaging::{SW_MINIMIZE, ShowWindow};
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_MINIMIZE);
        }
    }

    /// Forcibly shows a hidden window, used during daemon shutdown.
    ///
    /// Uses `SW_SHOW` for stronger activation than `SW_SHOWNOACTIVATE`,
    /// ensuring Windows fully restores the window's visibility state.
    pub fn force_show(&self) {
        use windows::Win32::UI::WindowsAndMessaging::{SW_SHOW, ShowWindow};
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
        }
    }

    /// Returns whether this window is maximized.
    pub fn is_maximized(&self) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::IsZoomed;
        unsafe { IsZoomed(self.hwnd).as_bool() }
    }

    /// Returns the owner window handle, or `None` if the window has no owner.
    ///
    /// Owned top-level windows are typically dialogs or property sheets
    /// spawned by an application's main window.
    pub fn owner(&self) -> Option<usize> {
        use windows::Win32::UI::WindowsAndMessaging::{GW_OWNER, GetWindow};
        unsafe {
            GetWindow(self.hwnd, GW_OWNER)
                .ok()
                .filter(|h| !h.is_invalid())
                .map(|h| h.0 as usize)
        }
    }

    /// Returns whether this looks like a real application window.
    ///
    /// Checks for a caption bar (`WS_CAPTION`), rejects tool windows
    /// (`WS_EX_TOOLWINDOW`), rejects owned windows, and rejects modal
    /// dialog frames (`WS_EX_DLGMODALFRAME`).
    ///
    /// The modal-frame check is important because when a dialog is
    /// created on a different thread from its owner, the owner
    /// relationship may not be queryable at the time
    /// `EVENT_OBJECT_CREATE` fires. `WS_EX_DLGMODALFRAME` is set
    /// during `CreateWindowEx` itself, so it's always available.
    pub fn is_app_window(&self) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::{
            GWL_EXSTYLE, GWL_STYLE, GW_OWNER, GetWindow, GetWindowLongPtrW, WS_CAPTION,
            WS_EX_DLGMODALFRAME, WS_EX_TOOLWINDOW,
        };

        unsafe {
            let style = GetWindowLongPtrW(self.hwnd, GWL_STYLE) as u32;
            let ex_style = GetWindowLongPtrW(self.hwnd, GWL_EXSTYLE) as u32;

            let has_caption = (style & WS_CAPTION.0) == WS_CAPTION.0;
            let is_tool = (ex_style & WS_EX_TOOLWINDOW.0) == WS_EX_TOOLWINDOW.0;
            let is_dialog = (ex_style & WS_EX_DLGMODALFRAME.0) != 0;
            let has_owner = GetWindow(self.hwnd, GW_OWNER)
                .is_ok_and(|owner| !owner.is_invalid());

            has_caption && !is_tool && !has_owner && !is_dialog
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
        Ok(crate::monitor::rect_from_win32(&frame))
    }

    fn set_rect(&self, rect: &mosaico_core::Rect) -> WindowResult<()> {
        // Compensate for invisible borders so the visible portion
        // lands exactly at the requested position and size.
        let border = frame::border_offset(self.hwnd)?;

        let x = rect.x - border.left;
        let y = rect.y - border.top;
        let cx = rect.width + border.left + border.right;
        let cy = rect.height + border.top + border.bottom;

        // SWP_NOSENDCHANGING suppresses WM_WINDOWPOSCHANGING so the
        // window cannot reject or modify the requested dimensions.
        //
        // SWP_NOCOPYBITS discards old client-area contents instead
        // of blitting them, avoiding stale surfaces.
        let mut flags = SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOCOPYBITS;

        // GPU-composited apps (Chrome, Electron) need SWP_FRAMECHANGED
        // to trigger WM_NCCALCSIZE, which makes them recalculate their
        // client area and update the rendering surface. However,
        // SWP_FRAMECHANGED also lets windows enforce size constraints
        // via WM_NCCALCSIZE, so we only add it for Chromium-based
        // windows that are known to need it.
        let frame_changed = self.needs_frame_changed();
        if frame_changed {
            flags |= SWP_FRAMECHANGED;
        }

        let hwnd_val = self.hwnd.0 as usize;
        mosaico_core::log_debug!(
            "set_rect 0x{:X}: target({},{} {}x{}) border(L:{} T:{} R:{} B:{}) frame_changed={}",
            hwnd_val,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            border.left,
            border.top,
            border.right,
            border.bottom,
            frame_changed
        );

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
