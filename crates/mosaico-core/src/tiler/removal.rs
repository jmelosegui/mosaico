//! Taking windows out of the layout.

use crate::WindowId;

use super::{Plan, Tiler};

impl Tiler {
    /// Removes a window from the layout, hands focus to a sibling if it
    /// had it, and retiles.
    ///
    /// With `active_only`, a window on an inactive workspace is left
    /// alone.
    pub(super) fn remove_from_tiling(
        &mut self,
        plan: &mut Plan,
        window: WindowId,
        reason: &str,
        active_only: bool,
    ) {
        let Some((mon_idx, ws_idx)) = self.find_window(window) else {
            return;
        };
        let on_active = ws_idx == self.monitors[mon_idx].active_workspace;
        if active_only && !on_active {
            return;
        }

        // Out of the layout, it is no longer the tiler's to show again,
        // and its id may go to a new window.
        self.hidden_how.forget(window);
        let ws = &mut self.monitors[mon_idx].workspaces[ws_idx];
        ws.remove(window);
        crate::log_info!(
            "-{} {} from mon {} ws {} (now {})",
            reason,
            window,
            mon_idx,
            ws_idx + 1,
            ws.len()
        );
        if ws.monocle() && ws.monocle_window() == Some(window) {
            ws.set_monocle(false);
            ws.set_monocle_window(None);
        }

        if self.focused_window == Some(window) {
            self.focused_window = None;
            // Promote a sibling so the border stays visible.
            if on_active {
                let ws = &self.monitors[mon_idx].workspaces[ws_idx];
                let replacement = ws
                    .last_focused()
                    .filter(|&w| ws.contains(w))
                    .or_else(|| ws.windows().first().copied());
                if let Some(next) = replacement {
                    self.focus_window(plan, next, false);
                }
            }
        }

        if on_active {
            self.apply_layout_on(plan, mon_idx);
        }
    }
}
