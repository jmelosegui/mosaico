use std::mem;

use mosaico_core::WindowResult;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Dwm::{DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute};
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

/// The invisible border widths around a window.
///
/// On Windows 10/11, windows have invisible drop-shadow borders that
/// `GetWindowRect` includes but are not visually part of the window.
/// Typical values are ~7px left/right/bottom and 0px top.
pub struct BorderOffset {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// Returns the visible bounds of a window using DWM extended frame bounds.
///
/// Falls back to `GetWindowRect` if DWM is unavailable.
pub fn visible_rect(hwnd: HWND) -> WindowResult<RECT> {
    let mut frame = RECT::default();
    let result = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut frame as *mut RECT as *mut _,
            mem::size_of::<RECT>() as u32,
        )
    };

    if result.is_err() {
        unsafe { GetWindowRect(hwnd, &mut frame)? };
    }

    Ok(frame)
}

/// Computes the invisible border widths by comparing `GetWindowRect`
/// (includes borders) with `DWMWA_EXTENDED_FRAME_BOUNDS` (visible area).
pub fn border_offset(hwnd: HWND) -> WindowResult<BorderOffset> {
    let mut window_rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut window_rect)? };

    let frame_rect = visible_rect(hwnd)?;

    // If both rects are the same, there are no invisible borders.
    Ok(BorderOffset {
        left: frame_rect.left - window_rect.left,
        top: frame_rect.top - window_rect.top,
        right: window_rect.right - frame_rect.right,
        bottom: window_rect.bottom - frame_rect.bottom,
    })
}
