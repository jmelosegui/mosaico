use super::*;
use windows::Win32::UI::WindowsAndMessaging::{IsWindow, SendMessageW};

/// Creates a window of the real bar class, so a message sent to it
/// goes through the registered window procedure rather than calling
/// the function directly.
///
/// Returns `None` when the window cannot be created, so a machine
/// with no interactive desktop skips instead of failing.
fn bar_class_window() -> Option<HWND> {
    ensure_class_registered();

    // SAFETY: creates a small popup of the already registered bar
    // class. The caller destroys it.
    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT,
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
        )
    }
    .ok()?;

    (!hwnd.is_invalid()).then_some(hwnd)
}

#[test]
fn wm_close_does_not_destroy_the_bar() {
    // Arrange
    let Some(hwnd) = bar_class_window() else {
        return;
    };

    // Act -- the daemon thread pumps now, so this message reaches the
    // window procedure instead of sitting in the queue. End task and
    // anything else that closes a process's windows in bulk sends it.
    // SAFETY: sending a message to a window this thread created.
    unsafe {
        SendMessageW(hwnd, WM_CLOSE, None, None);
    }

    // SAFETY: reading the state of a window this thread owns.
    let alive = unsafe { IsWindow(Some(hwnd)).as_bool() };

    // SAFETY: destroying a window this thread created.
    unsafe {
        let _ = DestroyWindow(hwnd);
    }

    // Assert
    assert!(
        alive,
        "WM_CLOSE destroyed the bar, which BarManager still holds a handle to"
    );
}
