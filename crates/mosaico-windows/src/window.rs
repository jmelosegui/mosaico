mod kind;
mod place;
mod pre_place;
mod process;

use mosaico_core::{Rect, Result, WindowId};

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowTextLengthW, GetWindowTextW, IsWindow, IsWindowVisible, RealGetWindowClassW,
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
    pub(crate) fn new(hwnd: HWND) -> Self {
        Self { hwnd }
    }

    /// The window a core [`WindowId`] refers to.
    pub fn from_id(id: WindowId) -> Self {
        Self {
            hwnd: HWND(id.raw() as usize as *mut _),
        }
    }

    /// The core id of this window, which is its `HWND` value.
    pub fn id(&self) -> WindowId {
        WindowId::from_raw(self.hwnd.0 as usize as u64)
    }

    /// Returns the raw window handle.
    pub(crate) fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Returns whether the window handle is still valid (the window exists).
    pub fn is_valid(&self) -> bool {
        // SAFETY: IsWindow is a read-only query on a valid HWND.
        unsafe { IsWindow(Some(self.hwnd)).as_bool() }
    }

    /// Returns whether this window is maximized.
    pub fn is_maximized(&self) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::IsZoomed;
        // SAFETY: IsZoomed is a read-only query on a valid HWND.
        unsafe { IsZoomed(self.hwnd).as_bool() }
    }

    /// Returns whether this window is minimized (iconic).
    pub fn is_minimized(&self) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::IsIconic;
        // SAFETY: IsIconic is a read-only query on a valid HWND.
        unsafe { IsIconic(self.hwnd).as_bool() }
    }

    /// Returns the owner window handle, or `None` if the window has no owner.
    ///
    /// Owned top-level windows are typically dialogs or property sheets
    /// spawned by an application's main window.
    pub fn owner(&self) -> Option<WindowId> {
        use windows::Win32::UI::WindowsAndMessaging::{GW_OWNER, GetWindow};
        // SAFETY: GetWindow with GW_OWNER is a read-only query on a valid HWND.
        unsafe {
            GetWindow(self.hwnd, GW_OWNER)
                .ok()
                .filter(|h| !h.is_invalid())
                .map(|h| Window::new(h).id())
        }
    }

    /// Returns whether the window is cloaked by the OS (e.g. suspended UWP apps).
    ///
    /// `IsWindowVisible` returns `true` for OS-cloaked windows, so this
    /// check is needed to avoid allocating layout slots for invisible
    /// UWP windows like a suspended Settings app.
    pub fn is_cloaked(&self) -> bool {
        use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};

        let mut cloaked: u32 = 0;
        // SAFETY: DwmGetWindowAttribute reads a DWM property into a caller-owned buffer.
        let result = unsafe {
            DwmGetWindowAttribute(
                self.hwnd,
                DWMWA_CLOAKED,
                &mut cloaked as *mut u32 as *mut _,
                std::mem::size_of::<u32>() as u32,
            )
        };
        result.is_ok() && cloaked != 0
    }

    /// Returns the window title.
    pub fn title(&self) -> Result<String> {
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

    /// Returns the window class name.
    pub fn class(&self) -> Result<String> {
        // SAFETY: RealGetWindowClassW reads the window class name.
        // 256 is the maximum class name length in Win32.
        unsafe {
            let mut buffer = [0u16; 256];
            let length = RealGetWindowClassW(self.hwnd, &mut buffer);
            Ok(String::from_utf16_lossy(&buffer[..length as usize]))
        }
    }

    /// Returns the window's visible bounds, without the invisible resize
    /// borders.
    pub fn rect(&self) -> Result<Rect> {
        let frame = frame::visible_rect(self.hwnd)?;
        Ok(crate::monitor::rect_from_win32(&frame))
    }

    /// Returns whether the window is currently visible.
    pub fn is_visible(&self) -> bool {
        // SAFETY: IsWindowVisible is a simple query that returns a BOOL.
        unsafe { IsWindowVisible(self.hwnd).as_bool() }
    }
}
