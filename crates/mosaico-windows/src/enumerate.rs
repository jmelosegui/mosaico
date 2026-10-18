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
fn enumerate_windows() -> Result<Vec<WindowId>> {
    top_level_windows(should_include_window)
}

/// The top-level windows that `keep` accepts, topmost first.
///
/// This calls the Win32 `EnumWindows` API, which iterates over every top-level
/// window and invokes a callback for each one. We filter inside the callback
/// with `keep`, such as to keep only "real" application windows.
pub(crate) fn top_level_windows(keep: fn(HWND) -> bool) -> Result<Vec<WindowId>> {
    let mut walk = Walk {
        keep,
        windows: Vec::new(),
    };

    // SAFETY: EnumWindows calls our callback for each top-level window.
    // We pass a pointer to our Walk as LPARAM (user data). The callback
    // casts it back to &mut Walk to collect results. This is safe because
    // EnumWindows runs synchronously, so the Walk outlives the call.
    unsafe {
        EnumWindows(
            Some(enum_window_callback),
            LPARAM(&mut walk as *mut _ as isize),
        )
    }
    .or_platform("list windows")?;

    Ok(walk.windows)
}

/// What [`top_level_windows`] keeps, and the windows it kept so far.
struct Walk {
    keep: fn(HWND) -> bool,
    windows: Vec<WindowId>,
}

/// Callback invoked by `EnumWindows` for each top-level window.
///
/// Returns `TRUE` to continue enumeration, `FALSE` to stop.
///
/// # How Win32 callbacks work
///
/// Win32 can't call Rust closures directly. Instead, we use `extern "system"`
/// (the Windows calling convention) and pass data through an `LPARAM` — a
/// pointer-sized integer used as "user data". We cast our `Walk` pointer
/// into an `LPARAM` when calling `EnumWindows`, and cast it back here.
unsafe extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // SAFETY: lparam is a pointer to our Walk, cast from top_level_windows().
    let walk = unsafe { &mut *(lparam.0 as *mut Walk) };

    if (walk.keep)(hwnd) {
        walk.windows.push(Window::new(hwnd).id());
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
