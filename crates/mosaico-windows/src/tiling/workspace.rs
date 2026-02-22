//! Workspace switching: go-to and send-to operations.
//!
//! Manages hiding/showing windows when switching workspaces and
//! tracks programmatically hidden windows to prevent spurious
//! EVENT_OBJECT_HIDE removals.

use super::{TilingManager, Window};

impl TilingManager {
    /// Switches to workspace `n` (1-indexed) on the focused monitor.
    ///
    /// Hides windows on the current workspace, shows windows on the
    /// target, retiles, and focuses the first window.
    pub(super) fn goto_workspace(&mut self, n: u8) {
        let idx = (n - 1) as usize;
        let mon_idx = self.focused_monitor;
        let Some(mon) = self.monitors.get(mon_idx) else {
            return;
        };
        if mon.active_workspace == idx {
            return; // already there
        }

        // Mark current workspace windows as programmatically hidden
        // so EVENT_OBJECT_HIDE events are ignored for them.
        for &hwnd in mon.active_ws().handles() {
            self.hidden_by_switch.insert(hwnd);
            Window::from_raw(hwnd).hide();
        }

        // Switch active workspace.
        self.monitors[mon_idx].active_workspace = idx;

        // Show windows on the target workspace and unmark them.
        for &hwnd in self.monitors[mon_idx].active_ws().handles() {
            self.hidden_by_switch.remove(&hwnd);
            Window::from_raw(hwnd).show();
        }

        mosaico_core::log_info!(
            "goto-workspace {} on mon {} ({} windows)",
            n,
            mon_idx,
            self.monitors[mon_idx].active_ws().len()
        );

        self.apply_layout_on(mon_idx);

        // Focus the first window on the new workspace, or clear focus.
        if let Some(&hwnd) = self.monitors[mon_idx].active_ws().handles().first() {
            self.focus_and_update_border(hwnd);
        } else {
            self.focused_window = None;
            self.update_border();
        }
    }

    /// Sends the focused window to workspace `n` (1-indexed) on the
    /// same monitor.
    ///
    /// The window is hidden if the target workspace is not active.
    pub(super) fn send_to_workspace(&mut self, n: u8) {
        let target_ws = (n - 1) as usize;
        let Some(hwnd) = self.focused_window else {
            return;
        };
        let mon_idx = self.focused_monitor;
        let Some(mon) = self.monitors.get(mon_idx) else {
            return;
        };
        if mon.active_workspace == target_ws {
            return; // already on target workspace
        }
        if !mon.active_ws().contains(hwnd) {
            return; // focused window is not on the active workspace
        }
        let src_ws_num = mon.active_workspace + 1;

        // Remove from current workspace, add to target.
        self.monitors[mon_idx].active_ws_mut().remove(hwnd);
        self.monitors[mon_idx].workspaces[target_ws].add(hwnd);

        // Hide the window since it's moving to a non-visible workspace.
        self.hidden_by_switch.insert(hwnd);
        Window::from_raw(hwnd).hide();

        mosaico_core::log_info!(
            "send-to-workspace {} 0x{:X} on mon {} (src ws {} -> dst ws {})",
            n,
            hwnd,
            mon_idx,
            src_ws_num,
            n
        );

        self.apply_layout_on(mon_idx);

        // Focus the next window on the current workspace, or clear focus.
        if let Some(&next) = self.monitors[mon_idx].active_ws().handles().first() {
            self.focus_and_update_border(next);
        } else {
            self.focused_window = None;
            self.update_border();
        }
    }
}
