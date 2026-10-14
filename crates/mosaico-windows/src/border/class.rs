//! The window class every border overlay is created with.

use std::sync::Once;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, RegisterClassW, WM_CLOSE, WNDCLASSW,
};
use windows::core::PCWSTR;

static REGISTER_CLASS: Once = Once::new();
pub(super) const CLASS_NAME: &[u16] = &[
    b'M' as u16,
    b'o' as u16,
    b's' as u16,
    b'a' as u16,
    b'i' as u16,
    b'c' as u16,
    b'o' as u16,
    b'B' as u16,
    b'o' as u16,
    b'r' as u16,
    b'd' as u16,
    b'e' as u16,
    b'r' as u16,
    0,
];

pub(super) fn ensure_class_registered() {
    REGISTER_CLASS.call_once(|| {
        let wc = WNDCLASSW {
            lpfnWndProc: Some(border_wnd_proc),
            lpszClassName: PCWSTR(CLASS_NAME.as_ptr()),
            ..Default::default()
        };
        // SAFETY: RegisterClassW is called once (guarded by Once) to
        // register the border window class with a valid WNDCLASSW.
        unsafe {
            RegisterClassW(&wc);
        }
    });
}

unsafe extern "system" fn border_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // Borders are owned by the tiling manager and destroyed in Drop when
    // their window leaves the active workspace. Now that the owning
    // thread dispatches messages, DefWindowProcW would act on WM_CLOSE by
    // destroying the window behind the owner's back, leaving a dangling
    // handle. Ignore the request instead.
    if msg == WM_CLOSE {
        return LRESULT(0);
    }
    // SAFETY: DefWindowProcW is the default handler required by WNDPROC.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

#[cfg(test)]
#[path = "class_tests.rs"]
mod tests;
