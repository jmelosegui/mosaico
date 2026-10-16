//! What the thread of a test window does to its own window, as an
//! application does to its own: maximize it and minimize it. Each only
//! calls the window's own procedure, so the thread handles none of the
//! messages queued for it.

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_STYLE, GetWindowLongW, GetWindowPlacement, SW_MINIMIZE, SetWindowLongW, ShowWindow,
    WINDOWPLACEMENT, WS_MAXIMIZE,
};

/// Gives a window of the calling thread the maximized style. Its
/// placement is read first, which makes Windows keep where it is now as
/// where a restore puts it back, as it does for a window maximized the
/// usual way.
pub(super) fn maximize(hwnd: HWND) {
    let mut placement = WINDOWPLACEMENT {
        length: size_of::<WINDOWPLACEMENT>() as u32,
        ..Default::default()
    };
    // SAFETY: reads the placement of a window the calling thread owns,
    // into a struct on the stack, and changes its style.
    unsafe {
        let _ = GetWindowPlacement(hwnd, &mut placement);
        let style = GetWindowLongW(hwnd, GWL_STYLE);
        SetWindowLongW(hwnd, GWL_STYLE, style | WS_MAXIMIZE.0 as i32);
    }
}

/// Minimizes a window of the calling thread, which then comes back
/// maximized once restored if it was maximized.
pub(super) fn minimize(hwnd: HWND) {
    // SAFETY: changes the show state of a window the calling thread owns,
    // which is never active, so no other window is activated.
    unsafe {
        let _ = ShowWindow(hwnd, SW_MINIMIZE);
    }
}
