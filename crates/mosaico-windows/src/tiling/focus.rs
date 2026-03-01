//! Focus management and border overlay for the tiling manager.

use mosaico_core::window::Window as WindowTrait;
use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;

use crate::border::Color;
use crate::window::Window;

use super::TilingManager;

impl TilingManager {
    /// Moves the cursor to the center of the given window.
    fn move_cursor_to_window(hwnd: usize) {
        let Ok(rect) = Window::from_raw(hwnd).rect() else {
            return;
        };
        let cx = rect.x + rect.width / 2;
        let cy = rect.y + rect.height / 2;
        // SAFETY: SetCursorPos is safe to call with screen coordinates.
        unsafe {
            let _ = SetCursorPos(cx, cy);
        }
    }

    /// Sets the focused window, brings it to the foreground, and
    /// refreshes the focus border.
    pub(super) fn focus_and_update_border(&mut self, hwnd: usize) {
        self.focused_window = Some(hwnd);
        self.focused_maximized = Window::from_raw(hwnd).is_maximized();
        Window::from_raw(hwnd).set_foreground();
        if self.mouse_follows_focus && !self.focus_from_mouse {
            Self::move_cursor_to_window(hwnd);
        }
        self.focus_from_mouse = false;
        self.update_border();
    }

    pub(super) fn update_border(&self) {
        let Some(border) = &self.border else {
            return;
        };
        let Some(hwnd) = self.focused_window else {
            border.hide();
            return;
        };
        let window = Window::from_raw(hwnd);
        // Hide the border when the focused window is maximized —
        // the border would be behind the maximized window anyway and
        // trying to keep it topmost causes z-order flickering.
        if window.is_maximized() {
            border.hide();
            return;
        }
        let Ok(rect) = window.rect() else {
            return;
        };
        let Some(mon) = self.monitors.get(self.focused_monitor) else {
            return;
        };
        let is_monocle = mon.active_ws().monocle();
        let hex = if is_monocle {
            &self.border_config.monocle
        } else {
            &self.border_config.focused
        };
        let color = Color::from_hex(hex).unwrap_or(Color {
            r: 0,
            g: 0xB4,
            b: 0xD8,
        });
        border.show(
            &rect,
            color,
            self.border_config.width,
            self.border_config.corner_style.border_radius(),
            window.hwnd(),
        );
    }

    pub(super) fn hide_border(&self) {
        if let Some(border) = &self.border {
            border.hide();
        }
    }

    /// Re-positions the focus border to match the current window rect.
    ///
    /// Call after work areas change (e.g. bar adjustment at startup) so
    /// the border reflects the final window position, not the pre-adjustment one.
    pub fn refresh_border(&self) {
        self.update_border();
    }

    /// Focuses the first window on the primary monitor at startup.
    pub(super) fn focus_initial(&mut self) {
        if let Some(mon) = self.monitors.first()
            && let Some(&hwnd) = mon.active_ws().handles().first()
        {
            self.focus_and_update_border(hwnd);
        }
    }
}
