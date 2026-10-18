//! Windows of the tests' own, which are never shown, on the test's own
//! thread, or inside one of them. Those on a thread of their own are in
//! `test_threads.rs`.

use mosaico_core::WindowId;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, WNDCLASSW, WS_CHILD,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
};
use windows::core::PCWSTR;

use super::Window;

#[path = "test_takes.rs"]
mod takes;

pub(crate) use takes::Takes;

/// A 1x1 tool window of the test's own on the calling thread, never
/// shown, and destroyed when dropped.
pub(crate) struct HiddenWindow(HWND);

impl HiddenWindow {
    /// `None` on a machine with no desktop to create it on, so the tests
    /// skip rather than fail.
    pub(crate) fn new() -> Option<Self> {
        create().map(Self)
    }

    pub(crate) fn id(&self) -> WindowId {
        Window::new(self.0).id()
    }

    pub(crate) fn hwnd(&self) -> HWND {
        self.0
    }
}

impl Drop for HiddenWindow {
    fn drop(&mut self) {
        // SAFETY: destroying a window this thread created.
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}

/// Default handler, required because `WNDCLASSW` wants a
/// `extern "system"` function pointer rather than the raw import.
pub(crate) unsafe extern "system" fn test_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: DefWindowProcW is the default handler required by WNDPROC.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Creates a 1x1 tool window on the calling thread, which must destroy
/// it. Without `WS_VISIBLE` it is never shown.
pub(crate) fn create() -> Option<HWND> {
    create_taking(Takes::Moves)
}

/// Creates a window as `create` does, which takes its moves as `takes`.
pub(crate) fn create_taking(takes: Takes) -> Option<HWND> {
    let (name, proc) = takes.class(Some(test_wnd_proc));
    let class: Vec<u16> = name.encode_utf16().collect();
    // SAFETY: registers the class, which fails harmlessly once it
    // exists, and creates a window with valid arguments.
    let hwnd = unsafe {
        let wc = WNDCLASSW {
            lpfnWndProc: proc,
            lpszClassName: PCWSTR(class.as_ptr()),
            ..Default::default()
        };
        RegisterClassW(&wc);
        CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            PCWSTR(class.as_ptr()),
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
        )
    };
    hwnd.ok().filter(|hwnd| !hwnd.is_invalid())
}

/// Creates a 1x1 window inside a hidden one, both on the calling thread,
/// which must destroy the hidden one, and with it the child: with
/// `WS_VISIBLE` when `shown`, so it counts as shown, yet it never
/// appears, as its parent is never shown. Returns the parent, then the
/// child.
pub(crate) fn create_child(shown: bool) -> Option<(HWND, HWND)> {
    let parent = create()?;
    let (name, _) = Takes::Moves.class(None);
    let class: Vec<u16> = name.encode_utf16().collect();
    let style = if shown {
        WS_CHILD | WS_VISIBLE
    } else {
        WS_CHILD
    };
    // SAFETY: creates a child of a window this thread just created, of a
    // class `create` registered, and destroys that window if it fails.
    let child = unsafe {
        CreateWindowExW(
            WS_EX_NOACTIVATE,
            PCWSTR(class.as_ptr()),
            PCWSTR::null(),
            style,
            0,
            0,
            1,
            1,
            Some(parent),
            None,
            None,
            None,
        )
    };
    match child {
        Ok(child) if !child.is_invalid() => Some((parent, child)),
        _ => {
            // SAFETY: destroying a window this thread created.
            unsafe {
                let _ = DestroyWindow(parent);
            }
            None
        }
    }
}
