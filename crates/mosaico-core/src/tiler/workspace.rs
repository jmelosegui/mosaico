//! Workspace switching: go-to and send-to.

use crate::config::WorkspaceMode;

use crate::WindowId;

use super::{Plan, Tiler};

impl Tiler {
    /// Switches to workspace `n` (1-indexed).
    ///
    /// In per-monitor mode only the focused monitor switches. In global
    /// mode every monitor switches in lockstep, like virtual desktops,
    /// and focus stays on the monitor the user was on.
    pub(super) fn goto_workspace(&mut self, plan: &mut Plan, n: u8) {
        let idx = usize::from(n.saturating_sub(1));
        match self.workspace_mode {
            WorkspaceMode::PerMonitor => {
                let mon_idx = self.focused_monitor;
                self.goto_workspace_on(plan, mon_idx, idx, true);
            }
            WorkspaceMode::Global => {
                let focused = self.focused_monitor;
                for mon_idx in 0..self.monitors.len() {
                    self.goto_workspace_on(plan, mon_idx, idx, mon_idx == focused);
                }
            }
        }
    }

    /// Switches one monitor to workspace `idx`.
    ///
    /// Shows the target workspace, lays it out and, with `refocus`,
    /// focuses its remembered window, before hiding the workspace being
    /// left. Hiding first would hide the window that holds focus, and
    /// the OS may show it again on its own during rapid switching if
    /// nothing else takes focus first. A no-op when already there.
    fn goto_workspace_on(&mut self, plan: &mut Plan, mon_idx: usize, idx: usize, refocus: bool) {
        let Some(mon) = self.monitors.get(mon_idx) else {
            return;
        };
        if mon.active_workspace == idx || idx >= mon.workspaces.len() {
            return;
        }

        // Remember what was focused here, for coming back. Only
        // meaningful on the focused monitor; the others remember nothing.
        let last_focused = if refocus { self.focused_window } else { None };
        let mon = &mut self.monitors[mon_idx];
        mon.active_ws_mut().set_last_focused(last_focused);
        let leaving: Vec<WindowId> = mon.active_ws().windows().to_vec();
        let prev_ws = mon.active_workspace;
        mon.active_workspace = idx;

        let how = self.layout.hiding;
        for &window in self.monitors[mon_idx].active_ws().windows() {
            self.hidden_how.show(plan, window, how);
        }

        self.apply_layout_on(plan, mon_idx);

        if refocus {
            // Prefer the monocle window, then the remembered one, then
            // the first window.
            let ws = self.monitors[mon_idx].active_ws();
            let present = |w: &WindowId| ws.contains(*w);
            let remembered = ws.last_focused().filter(present);
            let target = if ws.monocle() {
                ws.monocle_window().filter(present).or(remembered)
            } else {
                remembered
            }
            .or_else(|| ws.windows().first().copied());

            if let Some(window) = target {
                self.focus_window(plan, window, false);
            } else {
                self.focused_window = None;
                self.update_border(plan);
            }
        }

        for window in leaving {
            self.hidden_how.hide(plan, window, how);
        }

        // An empty workspace has nothing able to hold focus, so pin the
        // monitor until something resolves it. Only on the monitor the
        // user went to: in global mode the others just follow along.
        if refocus {
            self.park_focus_if_empty(plan);
        }

        crate::log_debug!(
            "goto-workspace {} on mon {} (from ws {}, {} windows)",
            idx + 1,
            mon_idx,
            prev_ws + 1,
            self.monitors[mon_idx].active_ws().len()
        );
    }

    /// Sends the focused window to workspace `n` (1-indexed) on the same
    /// monitor and follows it there. In global mode every other monitor
    /// switches to `n` too.
    pub(super) fn send_to_workspace(&mut self, plan: &mut Plan, n: u8) {
        let target_ws = usize::from(n.saturating_sub(1));
        let Some(window) = self.focused_window else {
            return;
        };
        let mon_idx = self.focused_monitor;
        let Some(mon) = self.monitors.get(mon_idx) else {
            return;
        };
        if mon.active_workspace == target_ws
            || target_ws >= mon.workspaces.len()
            || !mon.active_ws().contains(window)
        {
            return;
        }
        let src_ws_num = mon.active_workspace + 1;
        let how = self.layout.hiding;

        let mon = &mut self.monitors[mon_idx];
        mon.active_ws_mut().remove(window);
        mon.workspaces[target_ws].add(window);
        for &other in mon.active_ws().windows() {
            self.hidden_how.hide(plan, other, how);
        }
        mon.active_workspace = target_ws;
        for &shown in self.monitors[mon_idx].active_ws().windows() {
            self.hidden_how.show(plan, shown, how);
        }

        crate::log_info!(
            "send-to-workspace {} {} on mon {} (ws {} -> ws {})",
            n,
            window,
            mon_idx,
            src_ws_num,
            n
        );

        self.apply_layout_on(plan, mon_idx);
        self.focus_window(plan, window, false);

        if self.workspace_mode == WorkspaceMode::Global {
            for other in (0..self.monitors.len()).filter(|&i| i != mon_idx) {
                self.goto_workspace_on(plan, other, target_ws, false);
            }
        }
    }
}
