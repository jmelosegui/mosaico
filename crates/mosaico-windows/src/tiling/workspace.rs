//! Workspace switching: go-to and send-to operations.
//!
//! Manages hiding/showing windows when switching workspaces and
//! tracks programmatically hidden windows to prevent spurious
//! `EVENT_OBJECT_HIDE` removals.

use mosaico_core::config::HidingBehaviour;

use super::TilingManager;

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

        // Hide current workspace windows. Only guard with
        // hidden_by_switch for strategies that fire events.
        for &hwnd in mon.active_ws().handles() {
            if self.hiding != HidingBehaviour::Cloak {
                self.hidden_by_switch.insert(hwnd);
            }
            self.hide_window(hwnd);
        }

        // Switch active workspace.
        self.monitors[mon_idx].active_workspace = idx;

        // Show windows on the target workspace and unmark them.
        for &hwnd in self.monitors[mon_idx].active_ws().handles() {
            if self.hiding != HidingBehaviour::Cloak {
                self.hidden_by_switch.remove(&hwnd);
            }
            self.show_window(hwnd);
        }

        mosaico_core::log_info!(
            "goto-workspace {} on mon {} ({} windows)",
            n,
            mon_idx,
            self.monitors[mon_idx].active_ws().len()
        );

        self.apply_layout_on(mon_idx);

        // When returning to a monocle workspace, restore focus to the
        // remembered monocle window instead of the first in the list.
        let ws = self.monitors[mon_idx].active_ws();
        let target = if ws.monocle() {
            ws.monocle_window()
                .filter(|&h| ws.contains(h))
                .or_else(|| ws.handles().first().copied())
        } else {
            ws.handles().first().copied()
        };

        if let Some(hwnd) = target {
            self.focus_and_update_border(hwnd);
        } else {
            self.focused_window = None;
            self.update_border();
        }
    }

    /// Sends the focused window to workspace `n` (1-indexed) on the
    /// same monitor, then follows it there.
    ///
    /// Moves the window to the target workspace, switches to that
    /// workspace, and focuses the moved window.
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

        // Hide remaining windows on the source workspace.
        for &h in self.monitors[mon_idx].active_ws().handles() {
            if self.hiding != HidingBehaviour::Cloak {
                self.hidden_by_switch.insert(h);
            }
            self.hide_window(h);
        }

        // Switch to the target workspace.
        self.monitors[mon_idx].active_workspace = target_ws;

        // Show all windows on the target workspace.
        for &h in self.monitors[mon_idx].active_ws().handles() {
            if self.hiding != HidingBehaviour::Cloak {
                self.hidden_by_switch.remove(&h);
            }
            self.show_window(h);
        }

        mosaico_core::log_info!(
            "send-to-workspace {} 0x{:X} on mon {} (ws {} -> ws {})",
            n,
            hwnd,
            mon_idx,
            src_ws_num,
            n
        );

        self.apply_layout_on(mon_idx);
        self.focus_and_update_border(hwnd);
    }
}
