use mosaico_core::WindowResult;

use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, IsIconic, IsWindowVisible};
use windows::core::BOOL;

use crate::window::Window;

/// Enumerates all visible, non-minimized, top-level application windows.
///
/// This calls the Win32 `EnumWindows` API, which iterates over every top-level
/// window and invokes a callback for each one. We filter inside the callback
/// to keep only "real" application windows.
pub fn enumerate_windows() -> WindowResult<Vec<Window>> {
    let mut windows: Vec<Window> = Vec::new();

    // SAFETY: EnumWindows calls our callback for each top-level window.
    // We pass a pointer to our Vec as LPARAM (user data). The callback
    // casts it back to &mut Vec<Window> to collect results. This is safe
    // because EnumWindows runs synchronously — the Vec outlives the call.
    unsafe {
        EnumWindows(
            Some(enum_window_callback),
            LPARAM(&mut windows as *mut _ as isize),
        )?;
    }

    Ok(windows)
}

/// Callback invoked by `EnumWindows` for each top-level window.
///
/// Returns `TRUE` to continue enumeration, `FALSE` to stop.
///
/// # How Win32 callbacks work
///
/// Win32 can't call Rust closures directly. Instead, we use `extern "system"`
/// (the Windows calling convention) and pass data through an `LPARAM` — a
/// pointer-sized integer used as "user data". We cast our `Vec<Window>` pointer
/// into an `LPARAM` when calling `EnumWindows`, and cast it back here.
unsafe extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // SAFETY: lparam is a pointer to our Vec<Window>, cast from enumerate_windows().
    let windows = unsafe { &mut *(lparam.0 as *mut Vec<Window>) };

    if should_include_window(hwnd) {
        windows.push(Window::new(hwnd));
    }

    BOOL(1) // TRUE — continue enumerating
}

/// Determines whether a window should be included in the enumeration.
///
/// Filters out invisible, minimized, and non-application windows.
fn should_include_window(hwnd: HWND) -> bool {
    // SAFETY: These are simple query functions that read window state.
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return false;
        }
        if IsIconic(hwnd).as_bool() {
            return false;
        }
    }

    Window::new(hwnd).is_app_window()
}
