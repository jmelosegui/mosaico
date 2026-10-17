//! The desktop as the tiler reads it, through Win32 queries.

use mosaico_core::tiler::{Desktop, WindowFacts, WindowIdentity};
use mosaico_core::{MonitorId, Rect, WindowId};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

use crate::monitor;
use crate::window::Window;

/// Read-only Win32 view of windows, monitors and the cursor.
pub(super) struct Win32Desktop {
    /// Whether mosaico itself runs elevated.
    ///
    /// When it does not, `SetWindowPos` silently fails on windows of
    /// elevated processes across the UIPI boundary, so those windows are
    /// reported as out of reach.
    self_elevated: bool,
}

impl Win32Desktop {
    pub(super) fn new() -> Self {
        Self {
            self_elevated: crate::process::is_current_process_elevated(),
        }
    }

    pub(super) fn self_elevated(&self) -> bool {
        self.self_elevated
    }
}

impl Desktop for Win32Desktop {
    fn is_visible(&self, window: WindowId) -> bool {
        Window::from_id(window).is_visible()
    }

    fn is_minimized(&self, window: WindowId) -> bool {
        Window::from_id(window).is_minimized()
    }

    fn is_maximized(&self, window: WindowId) -> bool {
        Window::from_id(window).is_maximized()
    }

    fn rect(&self, window: WindowId) -> Option<Rect> {
        Window::from_id(window).rect().ok()
    }

    fn owner(&self, window: WindowId) -> Option<WindowId> {
        Window::from_id(window).owner()
    }

    /// The process that owns the window, and its class.
    fn identity(&self, window: WindowId) -> Option<WindowIdentity> {
        let win = Window::from_id(window);
        let process = win.process_id()?;
        let class = win.class().ok()?;
        Some(WindowIdentity { process, class })
    }

    fn monitor_of(&self, window: WindowId) -> MonitorId {
        monitor::monitor_id_for_window(window)
    }

    fn cursor(&self) -> Option<(i32, i32)> {
        let mut point = POINT::default();
        // SAFETY: GetCursorPos writes the cursor position into a stack
        // allocated POINT and does not modify any window state.
        unsafe { GetCursorPos(&mut point) }.ok()?;
        Some((point.x, point.y))
    }

    /// Walks the top-level windows, see `on_screen.rs`.
    fn windows_on_screen(&self) -> Vec<WindowId> {
        super::on_screen::windows_on_screen()
    }

    /// Rules out windows that are not application windows, cloaked
    /// windows (on another virtual desktop, or UWP frames that are not
    /// really there), and windows of elevated processes when mosaico is
    /// not elevated.
    fn candidate(&self, window: WindowId) -> Option<WindowFacts> {
        let win = Window::from_id(window);
        let has_caption = win.app_window_caption()?;
        if win.is_cloaked() {
            return None;
        }
        let class = win.class().unwrap_or_default();
        let title = win.title().unwrap_or_default();
        if !self.self_elevated && win.is_elevated() {
            mosaico_core::log_info!(
                "~skip {} [{}] \"{}\" (elevated, mosaico is not)",
                window,
                class,
                title,
            );
            return None;
        }
        Some(WindowFacts {
            class,
            title,
            path: win.exe_path(),
            has_caption,
        })
    }

    /// Reads what `candidate` reads, without ruling out cloaked windows,
    /// which is how `hiding = "cloak"` hides them, or windows of elevated
    /// processes, which the tiler only manages when it can position them.
    ///
    /// Unlike `candidate`, it first checks with `IsWindow` that the window
    /// still exists: the style queries behind `app_window_caption` read a
    /// closed window as an untitled application window rather than fail.
    fn facts(&self, window: WindowId) -> Option<WindowFacts> {
        let win = Window::from_id(window);
        if !win.is_valid() {
            return None;
        }
        Some(WindowFacts {
            has_caption: win.app_window_caption()?,
            class: win.class().unwrap_or_default(),
            title: win.title().unwrap_or_default(),
            path: win.exe_path(),
        })
    }

    /// The Windows animation setting, `SPI_GETCLIENTAREAANIMATION`, and
    /// never in a remote session (see `system_settings.rs`).
    fn animations_allowed(&self) -> bool {
        crate::system_settings::animations_allowed()
    }
}
