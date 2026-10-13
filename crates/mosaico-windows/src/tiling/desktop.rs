//! The desktop as the tiler reads it, through Win32 queries.

use mosaico_core::Rect;
use mosaico_core::tiler::{Desktop, WindowFacts};
use mosaico_core::window::Window as _;
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
    fn is_visible(&self, window: usize) -> bool {
        Window::from_raw(window).is_visible()
    }

    fn is_minimized(&self, window: usize) -> bool {
        Window::from_raw(window).is_minimized()
    }

    fn is_maximized(&self, window: usize) -> bool {
        Window::from_raw(window).is_maximized()
    }

    fn rect(&self, window: usize) -> Option<Rect> {
        Window::from_raw(window).rect().ok()
    }

    fn owner(&self, window: usize) -> Option<usize> {
        Window::from_raw(window).owner()
    }

    fn monitor_of(&self, window: usize) -> usize {
        monitor::monitor_id_for_window(window)
    }

    fn cursor(&self) -> Option<(i32, i32)> {
        let mut point = POINT::default();
        // SAFETY: GetCursorPos writes the cursor position into a stack
        // allocated POINT and does not modify any window state.
        unsafe { GetCursorPos(&mut point) }.ok()?;
        Some((point.x, point.y))
    }

    /// Rules out windows that are not application windows, cloaked
    /// windows (on another virtual desktop, or UWP frames that are not
    /// really there), and windows of elevated processes when mosaico is
    /// not elevated.
    fn candidate(&self, window: usize) -> Option<WindowFacts> {
        let win = Window::from_raw(window);
        let has_caption = win.app_window_caption()?;
        if win.is_cloaked() {
            return None;
        }
        let class = win.class().unwrap_or_default();
        let title = win.title().unwrap_or_default();
        if !self.self_elevated && win.is_elevated() {
            mosaico_core::log_info!(
                "~skip 0x{:X} [{}] \"{}\" (elevated, mosaico is not)",
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
}
