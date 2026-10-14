use mosaico_core::tiler::{Display, Enumerator, Insets, WindowDetails, WindowFacts};
use mosaico_core::{Result, WindowId};

use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, IsIconic, IsWindowVisible};
use windows::core::BOOL;

use crate::error::OrPlatform;
use crate::window::Window;
use crate::{frame, monitor};

/// Lists windows and monitors through Win32.
///
/// Windows come from `EnumWindows`, which walks the top-level windows in
/// Z order, topmost first, so the list runs front to back. Monitors come
/// from `EnumDisplayMonitors`, with keys from the display configuration.
///
/// The process must be per-monitor DPI aware before this is used, as
/// every mosaico process is once [`WindowsSetup`](crate::setup::WindowsSetup)
/// has set it up. Otherwise Windows scales work areas and
/// `GetWindowRect` to the system DPI but reports the DWM frame bounds,
/// which window bounds come from, in physical pixels. On a scaled display
/// work areas and window bounds then disagree, which breaks the contract,
/// and the frame in `details` means nothing.
pub struct WindowsEnumerator;

impl Enumerator for WindowsEnumerator {
    fn windows(&self) -> Result<Vec<WindowId>> {
        enumerate_windows()
    }

    fn connected_monitors(&self) -> Result<Vec<Display>> {
        monitor::enumerate_monitors()
    }

    /// The frame is the invisible resize border Windows 10 and 11 draw
    /// around most windows: the difference between `GetWindowRect` and
    /// the DWM extended frame bounds.
    fn details(&self, window: WindowId) -> WindowDetails {
        let win = Window::from_id(window);
        WindowDetails {
            facts: WindowFacts {
                class: win.class().unwrap_or_default(),
                title: win.title().unwrap_or_default(),
                path: win.exe_path(),
                has_caption: win.app_window_caption() == Some(true),
            },
            rect: win.rect().ok(),
            minimized: win.is_minimized(),
            maximized: win.is_maximized(),
            monitor: monitor::monitor_id_for_window(window),
            frame: frame::border_offset(win.hwnd()).ok().map(|b| Insets {
                left: b.left,
                top: b.top,
                right: b.right,
                bottom: b.bottom,
            }),
        }
    }
}

/// Enumerates all visible, non-minimized, top-level application windows.
///
/// This calls the Win32 `EnumWindows` API, which iterates over every top-level
/// window and invokes a callback for each one. We filter inside the callback
/// to keep only "real" application windows.
fn enumerate_windows() -> Result<Vec<WindowId>> {
    let mut windows: Vec<WindowId> = Vec::new();

    // SAFETY: EnumWindows calls our callback for each top-level window.
    // We pass a pointer to our Vec as LPARAM (user data). The callback
    // casts it back to &mut Vec<WindowId> to collect results. This is safe
    // because EnumWindows runs synchronously — the Vec outlives the call.
    unsafe {
        EnumWindows(
            Some(enum_window_callback),
            LPARAM(&mut windows as *mut _ as isize),
        )
    }
    .or_platform("list windows")?;

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
/// pointer-sized integer used as "user data". We cast our `Vec<WindowId>` pointer
/// into an `LPARAM` when calling `EnumWindows`, and cast it back here.
unsafe extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // SAFETY: lparam is a pointer to our Vec<WindowId>, cast from enumerate_windows().
    let windows = unsafe { &mut *(lparam.0 as *mut Vec<WindowId>) };

    if should_include_window(hwnd) {
        windows.push(Window::new(hwnd).id());
    }

    BOOL(1) // TRUE — continue enumerating
}

/// Determines whether a window should be included in the enumeration.
///
/// Filters out invisible, minimized, and non-application windows.
/// Windows without a caption are kept so window rules can opt them in.
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

    Window::new(hwnd).app_window_caption().is_some()
}
