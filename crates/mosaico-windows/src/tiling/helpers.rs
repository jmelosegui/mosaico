//! Window lookup and management helpers for the tiling manager.

use mosaico_core::window::Window as WindowTrait;

use crate::monitor;
use crate::window::Window;

use super::TilingManager;

impl TilingManager {
    pub(super) fn is_tileable(&self, hwnd: usize) -> bool {
        let window = Window::from_raw(hwnd);
        if !window.is_visible() || !window.is_app_window() {
            return false;
        }
        let class = window.class().unwrap_or_default();
        let title = window.title().unwrap_or_default();
        mosaico_core::config::should_manage(&class, &title, &self.rules)
    }

    /// Re-assigns a window to the correct monitor after it was moved.
    ///
    /// If the window moved to a different monitor, it is removed from
    /// the old workspace and added to the new one, then both monitors
    /// are re-tiled.
    pub(super) fn reassign_monitor(&mut self, hwnd: usize) {
        let old = self.owning_monitor(hwnd);
        let new = self.monitor_index_for(hwnd);

        match (old, new) {
            (Some(from), Some(to)) if from != to => {
                self.monitors[from].active_ws_mut().remove(hwnd);
                self.monitors[to].active_ws_mut().add(hwnd);
                self.apply_layout_on(from);
                self.apply_layout_on(to);
            }
            (Some(idx), Some(_)) => {
                // Same monitor — just re-tile to snap it back.
                self.apply_layout_on(idx);
            }
            (None, Some(to)) if self.is_tileable(hwnd) => {
                // Window wasn't tracked but appeared on a monitor.
                self.monitors[to].active_ws_mut().add(hwnd);
                self.apply_layout_on(to);
            }
            _ => {}
        }
    }

    pub(super) fn monitor_index_for(&self, hwnd: usize) -> Option<usize> {
        let mid = monitor::monitor_id_for_window(hwnd);
        self.monitors.iter().position(|m| m.id == mid)
    }

    pub(super) fn is_managed_on_active_workspace(&self, hwnd: usize) -> bool {
        self.monitors
            .get(self.focused_monitor)
            .is_some_and(|mon| mon.active_ws().contains(hwnd))
    }

    pub(super) fn owning_monitor(&self, hwnd: usize) -> Option<usize> {
        self.monitors
            .iter()
            .position(|m| m.workspaces.iter().any(|ws| ws.contains(hwnd)))
    }

    /// Finds which monitor and workspace contain the given window.
    ///
    /// Returns `(monitor_index, workspace_index)` or `None` if the
    /// window is not managed anywhere.
    pub(super) fn find_window(&self, hwnd: usize) -> Option<(usize, usize)> {
        for (mi, mon) in self.monitors.iter().enumerate() {
            for (wi, ws) in mon.workspaces.iter().enumerate() {
                if ws.contains(hwnd) {
                    return Some((mi, wi));
                }
            }
        }
        None
    }

    pub(super) fn close_focused(&mut self) {
        let Some(hwnd) = self.focused_window else {
            return;
        };
        // Send WM_CLOSE so the window can handle its own shutdown gracefully.
        use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};
        unsafe {
            let _ = PostMessageW(
                Some(windows::Win32::Foundation::HWND(hwnd as *mut _)),
                WM_CLOSE,
                windows::Win32::Foundation::WPARAM(0),
                windows::Win32::Foundation::LPARAM(0),
            );
        }
    }

    pub(super) fn minimize_focused(&mut self) {
        let Some(hwnd) = self.focused_window else {
            return;
        };
        // ShowWindow(SW_MINIMIZE) fires EVENT_SYSTEM_MINIMIZESTART which
        // the event loop translates to WindowEvent::Minimized. That event
        // removes the window from the active workspace and re-tiles.
        Window::from_raw(hwnd).minimize();
    }
}
