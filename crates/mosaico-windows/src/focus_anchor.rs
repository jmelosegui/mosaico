//! A hidden focus-holder window used to anchor the foreground to a
//! specific monitor when the active workspace on that monitor is empty.
//!
//! Win32 tracks a single "foreground window" globally, and transient
//! overlays like PowerToys Run restore focus to whatever was foreground
//! before they opened.  When mosaico switches focus to a monitor whose
//! workspace has no windows, there is nothing to take foreground — so
//! the previous monitor's window keeps it, and newly spawned apps land
//! on the wrong monitor.
//!
//! This module creates a single 1x1 layered popup window that can be
//! repositioned onto any monitor and set as foreground, giving Windows
//! a real target for foreground tracking without being visible to the user.

use std::sync::Once;

use mosaico_core::{Rect, WindowResult};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, HWND_TOP, LWA_ALPHA, RegisterClassW, SW_HIDE,
    SW_SHOWNA, SWP_NOSIZE, SetForegroundWindow, SetLayeredWindowAttributes, SetWindowPos,
    ShowWindow, WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_POPUP,
};
use windows::core::PCWSTR;

/// A 1x1 invisible popup window used as a foreground anchor.
pub struct FocusAnchor {
    hwnd: HWND,
}

static REGISTER_CLASS: Once = Once::new();
const CLASS_NAME: &[u16] = &[
    b'M' as u16,
    b'o' as u16,
    b's' as u16,
    b'a' as u16,
    b'i' as u16,
    b'c' as u16,
    b'o' as u16,
    b'F' as u16,
    b'o' as u16,
    b'c' as u16,
    b'u' as u16,
    b's' as u16,
    b'A' as u16,
    b'n' as u16,
    b'c' as u16,
    b'h' as u16,
    b'o' as u16,
    b'r' as u16,
    0,
];

fn ensure_class_registered() {
    REGISTER_CLASS.call_once(|| {
        let wc = WNDCLASSW {
            lpfnWndProc: Some(anchor_wnd_proc),
            lpszClassName: PCWSTR(CLASS_NAME.as_ptr()),
            ..Default::default()
        };
        // SAFETY: RegisterClassW is called once (guarded by Once) with a
        // valid WNDCLASSW whose string pointer is a static array.
        unsafe {
            RegisterClassW(&wc);
        }
    });
}

unsafe extern "system" fn anchor_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: DefWindowProcW is the default handler required by WNDPROC.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

impl FocusAnchor {
    /// Creates a hidden 1x1 layered anchor window.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the Win32 window could not be created.
    pub fn new() -> WindowResult<Self> {
        ensure_class_registered();

        let ex = WS_EX_LAYERED | WS_EX_TOOLWINDOW;
        // SAFETY: CreateWindowExW creates a popup window using the
        // registered class. The class name pointer is valid for the
        // lifetime of the process (static array).
        let hwnd = unsafe {
            CreateWindowExW(
                ex,
                PCWSTR(CLASS_NAME.as_ptr()),
                PCWSTR::null(),
                WS_POPUP,
                0,
                0,
                1,
                1,
                None,
                None,
                None,
                None,
            )?
        };

        // SAFETY: SetLayeredWindowAttributes makes the window fully
        // transparent. The HWND was just created above.
        unsafe {
            let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_ALPHA);
        }

        Ok(Self { hwnd })
    }

    /// Positions the anchor at the center of the given work area and
    /// sets it as the foreground window.
    ///
    /// Call when focus moves to a monitor whose active workspace is
    /// empty, so that subsequent new-window placement respects that
    /// monitor instead of the previous foreground monitor.
    pub fn activate_on(&self, work_area: &Rect) {
        let x = work_area.x + work_area.width / 2;
        let y = work_area.y + work_area.height / 2;
        // SAFETY: SetWindowPos, ShowWindow, and SetForegroundWindow are
        // safe to call with the anchor's valid HWND.
        unsafe {
            let _ = SetWindowPos(self.hwnd, Some(HWND_TOP), x, y, 0, 0, SWP_NOSIZE);
            let _ = ShowWindow(self.hwnd, SW_SHOWNA);
            let _ = SetForegroundWindow(self.hwnd);
        }
    }

    /// Hides the anchor.  Called when a real window takes focus again.
    pub fn hide(&self) {
        // SAFETY: ShowWindow is safe to call with the anchor's valid HWND.
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    /// Returns the underlying HWND as a `usize`, for comparison with
    /// hwnds reported by Win32 events.
    pub fn hwnd(&self) -> usize {
        self.hwnd.0 as usize
    }
}

impl Drop for FocusAnchor {
    fn drop(&mut self) {
        // SAFETY: DestroyWindow is safe to call with a valid HWND we own.
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}
