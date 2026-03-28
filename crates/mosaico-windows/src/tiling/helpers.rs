//! Window lookup and management helpers for the tiling manager.

use mosaico_core::window::Window as WindowTrait;

use crate::frame;
use crate::monitor;
use crate::window::Window;

use super::TilingManager;

impl TilingManager {
    pub(super) fn is_tileable(&self, hwnd: usize) -> bool {
        let window = Window::from_raw(hwnd);
        if !window.is_visible() || !window.is_app_window() || window.is_cloaked() {
            return false;
        }
        // When mosaico runs as a regular user, SetWindowPos silently fails
        // on windows owned by elevated processes (UIPI). Skip them.
        if !self.self_elevated && window.is_elevated() {
            let title = window.title().unwrap_or_default();
            let class = window.class().unwrap_or_default();
            mosaico_core::log_info!(
                "~skip 0x{:X} [{}] \"{}\" (elevated, mosaico is not)",
                hwnd,
                class,
                title,
            );
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

    /// Adopts an untracked window into the tiling layout if it is tileable.
    ///
    /// Some applications (e.g. WPF-based apps like Visual Studio) never
    /// fire EVENT_OBJECT_CREATE for their main window. This method is
    /// called from fallback event handlers (TitleChanged, Focused) to
    /// pick up windows that the Created handler missed.
    pub(super) fn try_adopt(&mut self, hwnd: usize) {
        // Don't use adopt_rejected here — try_adopt is called from
        // TitleChanged (relaxed id_object filter) where child element
        // events share the parent hwnd and would poison the cache.
        if !self.is_tileable(hwnd) {
            return;
        }
        let Some(idx) = self.monitor_index_for(hwnd) else {
            return;
        };
        if !self.monitors[idx].active_ws_mut().add(hwnd) {
            return;
        }
        let w = Window::from_raw(hwnd);
        let title = w.title().unwrap_or_default();
        let class = w.class().unwrap_or_default();
        mosaico_core::log_info!(
            "+adopt 0x{:X} [{}] \"{}\" on mon {} ws {} (now {})",
            hwnd,
            class,
            title,
            idx,
            self.monitors[idx].active_workspace + 1,
            self.monitors[idx].active_ws().len()
        );
        frame::set_corner_preference(w.hwnd(), self.border_config.corner_style);
        self.apply_layout_on(idx);
    }

    /// Adds a window to the focused monitor's active workspace and focuses it.
    ///
    /// Shared by the `Created` and `Restored` event handlers.
    pub(super) fn add_and_focus(&mut self, hwnd: usize) {
        // Place new windows on the focused monitor so they appear
        // where the user is working, not wherever the OS spawns them.
        let idx = self.focused_monitor;
        if self.monitors.get(idx).is_some() && self.monitors[idx].active_ws_mut().add(hwnd) {
            let w = Window::from_raw(hwnd);
            let title = w.title().unwrap_or_default();
            let class = w.class().unwrap_or_default();
            mosaico_core::log_info!(
                "+add 0x{:X} [{}] \"{}\" to mon {} ws {} (now {})",
                hwnd,
                class,
                title,
                idx,
                self.monitors[idx].active_workspace + 1,
                self.monitors[idx].active_ws().len()
            );
            frame::set_corner_preference(w.hwnd(), self.border_config.corner_style);
            // Focus the new window before layout so monocle
            // mode sizes the correct window.
            self.focused_window = Some(hwnd);
            // In monocle mode the newest window becomes the
            // monocle target so it fills the work area.
            if self.monitors[idx].active_ws().monocle() {
                self.monitors[idx]
                    .active_ws_mut()
                    .set_monocle_window(Some(hwnd));
            }
            self.apply_layout_on(idx);
            self.focus_from_mouse = false;
            self.focus_and_update_border(hwnd);
        }
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
