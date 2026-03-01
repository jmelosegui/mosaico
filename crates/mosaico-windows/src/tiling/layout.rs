//! Layout application: BSP tiling and monocle mode.
//!
//! Computes window positions from the workspace handle list and
//! applies them via Win32. Skips windows whose position is already
//! correct to avoid unnecessary repaints.

use mosaico_core::Rect;
use mosaico_core::window::Window as WindowTrait;

use super::{TilingManager, Window};

impl TilingManager {
    pub(super) fn toggle_monocle(&mut self) {
        if self.monitors.is_empty() {
            return;
        }
        let idx = self.focused_monitor;
        let enabling = !self.monitors[idx].monocle;
        self.monitors[idx].monocle = enabling;
        if enabling {
            self.monitors[idx].monocle_window = self.focused_window;
        } else {
            self.monitors[idx].monocle_window = None;
        }
        self.apply_layout_on(idx);
        self.update_border();
    }

    pub(super) fn retile_all(&mut self) {
        for i in 0..self.monitors.len() {
            self.apply_layout_on(i);
        }
        self.update_border();
    }

    pub(super) fn apply_layout_on(&mut self, monitor_idx: usize) {
        self.prune_stale_handles(monitor_idx);
        self.applying_layout = true;

        let Some(state) = self.monitors.get(monitor_idx) else {
            self.applying_layout = false;
            return;
        };
        if state.monocle {
            // In monocle mode, the focused window fills the work area.
            if let Some(hwnd) = self.focused_window
                && state.active_ws().contains(hwnd)
            {
                let gap = self.layout.gap;
                let area = Rect::new(
                    state.work_area.x + gap,
                    state.work_area.y + gap,
                    (state.work_area.width - gap * 2).max(1),
                    (state.work_area.height - gap * 2).max(1),
                );
                let window = Window::from_raw(hwnd);
                if let Err(e) = window.set_rect(&area) {
                    eprintln!("Failed to position window 0x{hwnd:X}: {e}");
                }
                window.invalidate();
                self.applying_layout = false;
                return;
            }
        }
        let positions = state
            .active_ws()
            .compute_layout(&self.layout, &state.work_area);
        for (hwnd, rect) in &positions {
            let window = Window::from_raw(*hwnd);
            if window.rect().ok().as_ref() == Some(rect) {
                continue;
            }
            if let Err(e) = window.set_rect(rect) {
                eprintln!("Failed to position window 0x{hwnd:X}: {e}");
            }
            window.invalidate();
        }
        self.applying_layout = false;
    }

    /// Removes handles from the workspace that are no longer valid windows.
    ///
    /// Some windows (especially UWP and WinUI apps) may close without
    /// firing a reliable `EVENT_OBJECT_DESTROY`. This ensures stale
    /// handles don't leave phantom slots in the layout.
    fn prune_stale_handles(&mut self, monitor_idx: usize) {
        let Some(state) = self.monitors.get_mut(monitor_idx) else {
            return;
        };
        let stale: Vec<usize> = state
            .active_ws()
            .handles()
            .iter()
            .copied()
            .filter(|&hwnd| !Window::from_raw(hwnd).is_visible())
            .collect();
        for hwnd in stale {
            state.active_ws_mut().remove(hwnd);
        }
    }
}
