//! One monitor's bar: a layered Win32 popup window that can take the
//! pointer, but never activation.

use std::rc::Rc;
use std::sync::Once;

use mosaico_core::Rect;
use mosaico_core::bar::{BarId, PointerSink};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow, GWLP_USERDATA, RegisterClassW,
    SetWindowLongPtrW, WM_CLOSE, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT, WS_POPUP,
};
use windows::core::PCWSTR;

use super::input::BarInput;
use crate::error::OrPlatform;

/// A status bar overlay for a single monitor.
pub(super) struct Bar {
    pub(super) hwnd: HWND,
    pub(super) monitor_rect: Rect,
    /// What the window procedure reports the pointer with. Boxed, so the
    /// address the window keeps stays where it is, and kept until the
    /// window is gone (see `Drop`).
    pub(super) input: Box<BarInput>,
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
    b'B' as u16,
    b'a' as u16,
    b'r' as u16,
    0,
];

fn ensure_class_registered() {
    REGISTER_CLASS.call_once(|| {
        let wc = WNDCLASSW {
            lpfnWndProc: Some(bar_wnd_proc),
            lpszClassName: PCWSTR(CLASS_NAME.as_ptr()),
            ..Default::default()
        };
        // SAFETY: RegisterClassW is called once (guarded by Once) to
        // register the bar window class with a valid WNDCLASSW.
        unsafe {
            RegisterClassW(&wc);
        }
    });
}

unsafe extern "system" fn bar_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // The bar belongs to BarManager, which destroys it by dropping it. Now
    // that the owning thread dispatches messages, DefWindowProcW would act
    // on WM_CLOSE by destroying the window behind the owner's back, leaving
    // a dangling handle and a monitor with no bar until the next rebuild.
    // Anything that closes a process's windows in bulk, End task being
    // the common one, reaches this. Ignore the request instead.
    if msg == WM_CLOSE {
        return LRESULT(0);
    }
    // SAFETY: this is the window procedure of the bar class, handling one
    // message on the thread that made the window.
    if let Some(input) = unsafe { BarInput::of(hwnd) }
        && let Some(result) = input.handle(hwnd, msg, wparam, lparam)
    {
        return result;
    }
    // SAFETY: DefWindowProcW is the default handler required by WNDPROC.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

impl Bar {
    /// Creates a new hidden bar overlay positioned at the top of the
    /// given monitor work area, letting the pointer through. Once it takes
    /// the pointer, it reports what the pointer does on it to `sink`, as
    /// the bar `id`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the Win32 window could not be created.
    pub(super) fn new(
        id: BarId,
        monitor_work_area: Rect,
        sink: Rc<dyn PointerSink>,
    ) -> mosaico_core::Result<Self> {
        ensure_class_registered();

        // `WS_EX_NOACTIVATE`: a click on the bar leaves focus where it is.
        let ex = WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE;
        // SAFETY: CreateWindowExW creates a layered popup window for the
        // bar overlay. The class name pointer is valid for the process
        // lifetime (static array).
        let hwnd = unsafe {
            CreateWindowExW(
                ex,
                PCWSTR(CLASS_NAME.as_ptr()),
                PCWSTR::null(),
                WS_POPUP,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                0,
                0,
                None,
                None,
                None,
                None,
            )
        }
        .or_platform("create a bar window")?;

        let input = Box::new(BarInput::new(id, sink));
        // SAFETY: the window keeps the address of `input`, which the bar
        // owns and keeps until the window no longer has it (see `Drop`).
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, std::ptr::from_ref(&*input) as isize);
        }
        Ok(Self {
            hwnd,
            monitor_rect: monitor_work_area,
            input,
        })
    }
}

impl Drop for Bar {
    fn drop(&mut self) {
        // SAFETY: the window forgets `input` first, so nothing it handles
        // while it is destroyed reaches it, then DestroyWindow cleans up
        // the bar overlay. The HWND is exclusively owned by this struct
        // and not used after drop.
        unsafe {
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

#[cfg(test)]
#[path = "window_tests.rs"]
mod tests;
