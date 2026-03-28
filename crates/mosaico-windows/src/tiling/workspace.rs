//! Workspace switching: go-to and send-to operations.
//!
//! Manages hiding/showing windows when switching workspaces and
//! tracks programmatically hidden windows to prevent spurious
//! `EVENT_OBJECT_HIDE` removals.

use std::time::{Duration, Instant};

use mosaico_core::config::HidingBehaviour;

use super::TilingManager;

/// How long after a workspace switch to suppress focus-triggered switches.
///
/// Win32 events (e.g. `EVENT_SYSTEM_FOREGROUND`) fired during the switch are
/// queued asynchronously and may arrive after the switch completes. A 500 ms
/// cooldown is long enough to absorb these deferred events without being
/// noticeable to the user.
const WS_SWITCH_COOLDOWN: Duration = Duration::from_millis(500);

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

        self.ws_switch_cooldown = Some(Instant::now() + WS_SWITCH_COOLDOWN);

        // Remember which window was focused before leaving.
        self.monitors[mon_idx]
            .active_ws_mut()
            .set_last_focused(self.focused_window);

        // Collect the previous workspace's handles so we can hide them
        // AFTER the target workspace's window is foregrounded. Hiding
        // first would leave the old foreground window cloaked, and DWM
        // may auto-uncloak it during rapid switching if no new window
        // takes foreground before the next switch.
        let prev_handles: Vec<usize> = self.monitors[mon_idx].active_ws().handles().to_vec();
        let prev_ws = self.monitors[mon_idx].active_workspace;

        // Switch active workspace.
        self.monitors[mon_idx].active_workspace = idx;

        // Show windows on the target workspace and unmark them.
        for &hwnd in self.monitors[mon_idx].active_ws().handles() {
            if self.hiding != HidingBehaviour::Cloak {
                self.hidden_by_switch.remove(&hwnd);
            }
            self.show_window(hwnd);
        }

        self.apply_layout_on(mon_idx);

        // Restore focus to the last focused window on this workspace.
        // In monocle mode, prefer the monocle window. Fall back to the
        // first window if the remembered window is no longer present.
        let ws = self.monitors[mon_idx].active_ws();
        let target = if ws.monocle() {
            ws.monocle_window()
                .filter(|&h| ws.contains(h))
                .or_else(|| ws.last_focused().filter(|&h| ws.contains(h)))
                .or_else(|| ws.handles().first().copied())
        } else {
            ws.last_focused()
                .filter(|&h| ws.contains(h))
                .or_else(|| ws.handles().first().copied())
        };

        if let Some(hwnd) = target {
            self.focus_and_update_border(hwnd);
        } else {
            self.focused_window = None;
            self.update_border();
        }

        // Now hide the previous workspace's windows. Doing this AFTER
        // focusing the target window ensures the foreground has moved
        // away, preventing DWM from auto-uncloaking the old window.
        for &hwnd in &prev_handles {
            if self.hiding != HidingBehaviour::Cloak {
                self.hidden_by_switch.insert(hwnd);
            }
            self.hide_window(hwnd);
        }

        mosaico_core::log_debug!(
            "goto-workspace {} on mon {} (from ws {}, {} windows)",
            n,
            mon_idx,
            prev_ws + 1,
            self.monitors[mon_idx].active_ws().len()
        );
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

        self.ws_switch_cooldown = Some(Instant::now() + WS_SWITCH_COOLDOWN);

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
