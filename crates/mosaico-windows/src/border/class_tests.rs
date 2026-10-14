use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, IsWindow, SendMessageW, WS_EX_LAYERED, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT, WS_POPUP,
};

use super::*;

/// Creates a window of the real border class, so a message sent to
/// it goes through the registered window procedure.
///
/// Returns `None` when the window cannot be created, so a machine
/// with no interactive desktop skips instead of failing.
fn border_class_window() -> Option<HWND> {
    ensure_class_registered();

    // SAFETY: creates a small popup of the already registered border
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
fn wm_close_does_not_destroy_the_border() {
    // Arrange
    let Some(hwnd) = border_class_window() else {
        return;
    };

    // Act -- the daemon thread pumps now, so this message reaches the
    // window procedure instead of sitting in the queue.
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
        "WM_CLOSE destroyed the border, which the tiling manager still holds a handle to"
    );
}
