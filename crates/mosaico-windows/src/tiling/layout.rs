//! Layout application: BSP tiling and monocle mode.
//!
//! Computes window positions from the workspace handle list and
//! applies them via Win32. Skips windows whose position is already
//! correct to avoid unnecessary repaints.

use mosaico_core::window::Window as WindowTrait;
use mosaico_core::{BspLayout, LayoutKind, Rect, VerticalStackLayout};

use super::{TilingManager, Window};

impl TilingManager {
    pub(super) fn gap(&self) -> i32 {
        self.layout_gap
    }

    pub(super) fn ratio(&self) -> f64 {
        self.layout_ratio
    }

    /// Computes the layout positions for the active workspace on the given monitor.
    pub(super) fn compute_positions(&self, monitor_idx: usize) -> Vec<(usize, mosaico_core::Rect)> {
        let Some(state) = self.monitors.get(monitor_idx) else {
            return Vec::new();
        };
        let gap = self.gap();
        let ratio = self.ratio();
        match state.active_ws().layout_kind() {
            LayoutKind::Bsp => {
                let layout = BspLayout { gap, ratio };
                state.active_ws().compute_layout(&layout, &state.work_area)
            }
            LayoutKind::VerticalStack => {
                let layout = VerticalStackLayout { gap, ratio };
                state.active_ws().compute_layout(&layout, &state.work_area)
            }
        }
    }

    pub(super) fn cycle_layout(&mut self) {
        if self.monitors.is_empty() {
            return;
        }
        let idx = self.focused_monitor;
        let ws = self.monitors[idx].active_ws_mut();
        let next = ws.layout_kind().next();
        ws.set_layout_kind(next);
        self.apply_layout_on(idx);
    }

    pub(super) fn toggle_monocle(&mut self) {
        if self.monitors.is_empty() {
            return;
        }
        let idx = self.focused_monitor;
        let ws = self.monitors[idx].active_ws_mut();
        let enabling = !ws.monocle();
        ws.set_monocle(enabling);
        if enabling {
            ws.set_monocle_window(self.focused_window);
        } else {
            ws.set_monocle_window(None);
        }
        self.apply_layout_on(idx);
    }

    pub(super) fn retile_all(&mut self) {
        for i in 0..self.monitors.len() {
            self.apply_layout_on(i);
        }
    }

    pub(super) fn apply_layout_on(&mut self, monitor_idx: usize) {
        self.prune_stale_handles(monitor_idx);
        self.applying_layout = true;

        let Some(state) = self.monitors.get(monitor_idx) else {
            self.applying_layout = false;
            return;
        };
        if state.active_ws().monocle() {
            // In monocle mode, the remembered monocle window fills the
            // work area. Fall back to focused_window when monocle_window
            // is unset (e.g. toggled without a specific target).
            let monocle_hwnd = state.active_ws().monocle_window().or(self.focused_window);
            if let Some(hwnd) = monocle_hwnd
                && state.active_ws().contains(hwnd)
            {
                let gap = self.gap();
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
                self.update_border();
                return;
            }
        }
        let positions = self.compute_positions(monitor_idx);
        for (hwnd, rect) in &positions {
            let window = Window::from_raw(*hwnd);
            // Always call set_rect for maximized windows — it clears
            // WS_MAXIMIZE which SetWindowPos alone doesn't remove.
            if !window.is_maximized() && window.rect().ok().as_ref() == Some(rect) {
                continue;
            }
            if let Err(e) = window.set_rect(rect) {
                eprintln!("Failed to position window 0x{hwnd:X}: {e}");
            }
            window.invalidate();
        }
        self.applying_layout = false;
        self.update_border();
    }

    /// Removes handles from the workspace that are no longer valid windows.
    ///
    /// Some windows (especially UWP and WinUI apps) may close without
    /// firing a reliable `EVENT_OBJECT_DESTROY`. Similarly, a window may
    /// be minimized without firing `EVENT_SYSTEM_MINIMIZESTART` (e.g. it
    /// was minimized before being adopted, or the event was lost). This
    /// ensures stale or minimized handles don't leave phantom slots in
    /// the layout.
    fn prune_stale_handles(&mut self, monitor_idx: usize) {
        let Some(state) = self.monitors.get_mut(monitor_idx) else {
            return;
        };
        let stale: Vec<usize> = state
            .active_ws()
            .handles()
            .iter()
            .copied()
            .filter(|&hwnd| {
                let w = Window::from_raw(hwnd);
                !w.is_visible() || w.is_minimized()
            })
            .collect();
        for hwnd in &stale {
            state.active_ws_mut().remove(*hwnd);
            mosaico_core::log_info!("-prune 0x{:X} (stale or minimized)", hwnd);
        }
    }
}
