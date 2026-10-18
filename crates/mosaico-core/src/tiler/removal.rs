//! Taking windows out of the layout.

use crate::config::HidingBehaviour;
use crate::{WindowId, Workspace};

use super::{Effect, Plan, Tiler};

/// Takes `window` out of `ws` for good, and has what is still held back
/// for it dropped.
///
/// A move held for the window was decided while it was tiled, and must
/// not reach it now, whether or not it has a border to drop. Every way a
/// window leaves the layout goes through here: an event that takes it
/// out, rules that stop managing it, and the layout finding it gone or
/// minimized. Moving it to another workspace or monitor does not.
pub(super) fn untile(plan: &mut Plan, ws: &mut Workspace, window: WindowId) {
    if ws.remove(window) {
        plan.emit(Effect::Untile { window });
    }
}

impl Tiler {
    /// Removes a window from the layout, see [`untile`], hands focus to a
    /// sibling if it had it, and retiles.
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
        untile(plan, ws, window);
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

    /// Removes windows that are gone or minimized from the monitor's
    /// active workspace.
    ///
    /// Some applications close without reporting it, and a window can
    /// be minimized without a report, for example before it was
    /// adopted. Either would leave an empty slot in the layout.
    ///
    /// A window that is no longer shown is also forgotten as decorated,
    /// so a later window given its id does not get the system's corners
    /// back for it. A minimized one keeps the tiler's corners until it
    /// hides or closes. Either leaves the layout, see [`untile`].
    ///
    /// A window the tiler hid with "hide" is kept while it still exists
    /// and is still hidden: it is out of view by the tiler's doing, and
    /// the display change that brings it into view, by moving it or by
    /// switching its monitor's workspace, shows it after this retile
    /// (#119, #124).
    pub(super) fn prune_stale_handles(&mut self, plan: &mut Plan, monitor_idx: usize) {
        let Some(state) = self.monitors.get_mut(monitor_idx) else {
            return;
        };
        let stale: Vec<WindowId> = state
            .active_ws()
            .windows()
            .iter()
            .copied()
            .filter(|&w| self.hidden_how.still_hidden(plan, w) != Some(HidingBehaviour::Hide))
            .filter(|&w| !plan.is_visible(w) || plan.is_minimized(w))
            .collect();
        for window in stale {
            untile(plan, state.active_ws_mut(), window);
            if !plan.is_visible(window) {
                self.decorated.remove(&window);
            }
            crate::log_info!("-prune {} (stale or minimized)", window);
        }
    }
}
