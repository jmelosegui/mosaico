//! `put_behind` on windows of the test's own, which are never shown.
//! No other window is touched.

use mosaico_core::{WindowControl, WindowId};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GW_HWNDNEXT, GWL_EXSTYLE, GetWindow,
    GetWindowLongPtrW, RegisterClassW, WINDOW_EX_STYLE, WNDCLASSW, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};
use windows::core::PCWSTR;

use crate::window::Window;
use crate::window_control::Win32WindowControl;

/// Default handler, required because `WNDCLASSW` wants a
/// `extern "system"` function pointer rather than the raw import.
unsafe extern "system" fn test_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: DefWindowProcW is the default handler required by WNDPROC.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// A 1x1 tool window of the test's own, never shown, and destroyed when
/// dropped.
struct TestWindow(HWND);

impl TestWindow {
    /// Creates one, always on top if `on_top`. `None` on a machine with
    /// no desktop to create it on, so the tests skip rather than fail.
    fn new(on_top: bool) -> Option<Self> {
        let class: Vec<u16> = "MosaicoStackingTestWindow\0".encode_utf16().collect();
        let layer = if on_top {
            WS_EX_TOPMOST
        } else {
            WINDOW_EX_STYLE(0)
        };
        // SAFETY: registers the class, which fails harmlessly once it
        // exists, and creates a window with valid arguments. Without
        // `WS_VISIBLE` it is never shown. It is destroyed on this thread,
        // which created it, when dropped.
        let hwnd = unsafe {
            let wc = WNDCLASSW {
                lpfnWndProc: Some(test_wnd_proc),
                lpszClassName: PCWSTR(class.as_ptr()),
                ..Default::default()
            };
            RegisterClassW(&wc);
            CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | layer,
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
        hwnd.ok().filter(|h| !h.is_invalid()).map(Self)
    }

    fn id(&self) -> WindowId {
        Window::new(self.0).id()
    }

    /// Whether the window is always on top.
    fn on_top(&self) -> bool {
        // SAFETY: a read-only query on a window this test owns.
        let ex_style = unsafe { GetWindowLongPtrW(self.0, GWL_EXSTYLE) } as u32;
        ex_style & WS_EX_TOPMOST.0 != 0
    }

    /// The window directly behind this one.
    fn next(&self) -> Option<HWND> {
        next(self.0)
    }

    /// Whether this window is in front of `back`, however far.
    fn in_front_of(&self, back: &TestWindow) -> bool {
        std::iter::successors(self.next(), |&hwnd| next(hwnd)).any(|hwnd| hwnd == back.0)
    }
}

impl Drop for TestWindow {
    fn drop(&mut self) {
        // SAFETY: destroying a window this thread created.
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}

/// The window directly behind `hwnd`, if any.
fn next(hwnd: HWND) -> Option<HWND> {
    // SAFETY: a read-only query, which changes no window.
    unsafe { GetWindow(hwnd, GW_HWNDNEXT) }.ok()
}

/// Creates `other`, then `window`, which starts in front of it, and puts
/// `window` behind `other`.
fn put_behind(window_on_top: bool, other_on_top: bool) -> Option<(TestWindow, TestWindow)> {
    let other = TestWindow::new(other_on_top)?;
    let window = TestWindow::new(window_on_top)?;
    Win32WindowControl::default()
        .put_behind(window.id(), other.id())
        .expect("both windows exist");
    Some((window, other))
}

#[test]
fn a_window_goes_directly_behind_another_of_its_layer() {
    for on_top in [false, true] {
        // Arrange / Act
        let Some((window, other)) = put_behind(on_top, on_top) else {
            return;
        };

        // Assert
        assert_eq!(other.next(), Some(window.0));
        assert_eq!(window.on_top(), on_top);
        assert_eq!(other.on_top(), on_top);
    }
}

#[test]
fn a_window_put_behind_one_always_on_top_does_not_become_so() {
    // Arrange / Act
    let Some((window, other)) = put_behind(false, true) else {
        return;
    };

    // Assert -- inserted after `other`, Windows would have made it
    // always on top too. It is behind `other` anyway.
    assert!(!window.on_top(), "the window was made always on top");
    assert!(other.on_top());
    assert!(other.in_front_of(&window));
}

#[test]
fn a_window_always_on_top_put_behind_one_that_is_not_stays_so() {
    // Arrange / Act
    let Some((window, other)) = put_behind(true, false) else {
        return;
    };

    // Assert -- inserted after `other`, Windows would have taken it out
    // of the layer on top.
    assert!(window.on_top(), "the window is no longer always on top");
    assert!(!other.on_top());
    assert!(window.in_front_of(&other));
}
