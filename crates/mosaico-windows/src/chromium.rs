//! Workarounds for Chromium-based windows (Chrome, Edge, Electron).

use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, GetWindowRect, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
    SetWindowPos,
};
use windows::core::w;

/// Puts Chromium's GPU presentation window back over the client area.
///
/// Chromium presents its pixels through a child window, owned by the GPU
/// process, that always covers the parent's client area at its origin.
/// After a display change across monitors with different DPI (seen with
/// a KVM switch, where an emulated low-DPI display stands in while the
/// KVM is away) that child can be left at a stale offset. The browser
/// itself keeps the right geometry, so clicks and hover land where the
/// content should be, but the picture is drawn shifted and mostly off
/// the window. Chromium only resizes the child afterwards and never
/// moves it, so nothing short of moving it back recovers.
///
/// Returns whether the child had to be moved.
pub(crate) fn realign_gpu_child(parent: HWND) -> bool {
    // SAFETY: FindWindowExW only reads the window list; a missing child
    // returns an error, which means there is nothing to realign.
    let Ok(child) =
        (unsafe { FindWindowExW(Some(parent), None, w!("Intermediate D3D Window"), None) })
    else {
        return false;
    };

    let mut origin = POINT::default();
    let mut child_rect = RECT::default();
    // SAFETY: Both calls write into caller-owned structs for valid HWNDs.
    let read = unsafe {
        ClientToScreen(parent, &mut origin).as_bool()
            && GetWindowRect(child, &mut child_rect).is_ok()
    };
    if !read || (child_rect.left == origin.x && child_rect.top == origin.y) {
        return false;
    }

    mosaico_core::log_info!(
        "Realigning GPU surface of 0x{:X}: was at ({},{}), client at ({},{})",
        parent.0 as usize,
        child_rect.left,
        child_rect.top,
        origin.x,
        origin.y
    );
    // The child belongs to another process. SWP_ASYNCWINDOWPOS posts the
    // move instead of waiting for that process to handle it.
    let flags = SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS;
    // SAFETY: Moves a valid child window to its parent's client origin.
    unsafe { SetWindowPos(child, None, 0, 0, 0, 0, flags).is_ok() }
}
