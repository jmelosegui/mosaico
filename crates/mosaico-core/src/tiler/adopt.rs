//! Deciding which windows are managed, and taking them in.

use crate::config;

use crate::WindowId;

use super::{Effect, Plan, Tiler};

/// Whether a window that exists but has not been shown yet should be
/// moved into the slot it is going to occupy.
///
/// Monocle workspaces are included: `pre_place` asks `monocle_rect` for
/// them, so the destination is right either way.
pub(super) fn should_pre_place(tracked: bool, passes_rules: bool) -> bool {
    !tracked && passes_rules
}

impl Tiler {
    pub(super) fn is_tileable(&self, plan: &Plan, window: WindowId) -> bool {
        plan.is_visible(window) && self.passes_tiling_rules(plan, window)
    }

    /// Everything `is_tileable` checks except visibility, so a window
    /// that has been created but not shown yet can be judged.
    pub(super) fn passes_tiling_rules(&self, plan: &Plan, window: WindowId) -> bool {
        let Some(facts) = plan.candidate(window) else {
            return false;
        };
        if facts.has_caption {
            config::should_manage(&facts.class, &facts.title, &facts.path, &self.rules)
        } else {
            // No title bar: only tile when a rule explicitly asks for it.
            config::matching_rule(&facts.class, &facts.title, &facts.path, &self.rules)
                == Some(true)
        }
    }

    /// Moves a window to the workspace of the monitor it is now on,
    /// after the user dragged it, and retiles.
    pub(super) fn reassign_monitor(&mut self, plan: &mut Plan, window: WindowId) {
        let old = self.owning_monitor(window);
        let new = self.monitor_index_for(plan, window);

        match (old, new) {
            (Some(from), Some(to)) if from != to => {
                self.monitors[from].active_ws_mut().remove(window);
                self.monitors[to].active_ws_mut().add(window);
                self.apply_layout_on(plan, from);
                self.apply_layout_on(plan, to);
            }
            // Same monitor: retile to snap it back.
            (Some(idx), Some(_)) => self.apply_layout_on(plan, idx),
            (None, Some(to)) if self.is_tileable(plan, window) => {
                self.monitors[to].active_ws_mut().add(window);
                self.apply_layout_on(plan, to);
            }
            _ => {}
        }
    }

    pub(super) fn monitor_index_for(&self, plan: &Plan, window: WindowId) -> Option<usize> {
        let id = plan.monitor_of(window);
        self.monitors.iter().position(|m| m.id == id)
    }

    pub(super) fn is_managed_on_active_workspace(&self, window: WindowId) -> bool {
        self.monitors
            .get(self.focused_monitor)
            .is_some_and(|mon| mon.active_ws().contains(window))
    }

    pub(super) fn owning_monitor(&self, window: WindowId) -> Option<usize> {
        self.find_window(window).map(|(mi, _)| mi)
    }

    /// Finds the `(monitor, workspace)` indices holding a window.
    pub(super) fn find_window(&self, window: WindowId) -> Option<(usize, usize)> {
        self.monitors.iter().enumerate().find_map(|(mi, mon)| {
            mon.workspaces
                .iter()
                .position(|ws| ws.contains(window))
                .map(|wi| (mi, wi))
        })
    }

    /// Whether the window is on the active workspace of its monitor.
    pub(super) fn is_on_active_workspace(&self, window: WindowId) -> bool {
        self.find_window(window)
            .is_some_and(|(mi, wi)| self.monitors[mi].active_workspace == wi)
    }

    /// Adopts a window we never heard being created, if it is tileable,
    /// onto the monitor it is on.
    pub(super) fn try_adopt(&mut self, plan: &mut Plan, window: WindowId) {
        // Rejections are not cached here: this runs for title changes,
        // which some applications report for child elements under the
        // parent window before the parent is ready.
        if !self.is_tileable(plan, window) {
            return;
        }
        let Some(idx) = self.monitor_index_for(plan, window) else {
            return;
        };
        if !self.monitors[idx].active_ws_mut().add(window) {
            return;
        }
        self.log_added("+adopt", plan, window, idx);
        self.decorate(plan, window);
        self.apply_layout_on(plan, idx);
    }

    /// Adds a window to the focused monitor's active workspace, so new
    /// windows appear where the user is working, and focuses it.
    pub(super) fn add_and_focus(&mut self, plan: &mut Plan, window: WindowId) {
        let idx = self.focused_monitor;
        if self.monitors.get(idx).is_none() || !self.monitors[idx].active_ws_mut().add(window) {
            return;
        }
        self.log_added("+add", plan, window, idx);
        self.decorate(plan, window);
        // Focus before the layout so monocle sizes the right window,
        // and make the newest window the monocle target.
        self.focused_window = Some(window);
        if self.monitors[idx].active_ws().monocle() {
            self.monitors[idx]
                .active_ws_mut()
                .set_monocle_window(Some(window));
        }
        self.apply_layout_on(plan, idx);
        self.focus_window(plan, window, false);
    }

    fn log_added(&self, verb: &str, plan: &Plan, window: WindowId, idx: usize) {
        let facts = plan.candidate(window).unwrap_or_default();
        crate::log_info!(
            "{} {} [{}] \"{}\" on mon {} ws {} (now {})",
            verb,
            window,
            facts.class,
            facts.title,
            idx,
            self.monitors[idx].active_workspace + 1,
            self.monitors[idx].active_ws().len()
        );
    }

    /// Moves a window that has been created but not shown yet into the
    /// slot it would get, so its first frame is already in place.
    ///
    /// Nothing is adopted and the workspace is left exactly as it was:
    /// the destination comes from asking the layout where the window
    /// would go, and only that window is moved. It is taken over for
    /// real when it is shown.
    pub(super) fn pre_place(&mut self, plan: &mut Plan, window: WindowId) {
        let idx = self.focused_monitor;
        let Some(state) = self.monitors.get(idx) else {
            return;
        };

        let rect = if state.active_ws().monocle() {
            // A window adopted onto a monocle workspace becomes the
            // monocle target, which always fills the work area.
            self.monocle_rect(idx)
        } else {
            if !self.monitors[idx].active_ws_mut().add(window) {
                return;
            }
            let positions = self.compute_positions(idx);
            self.monitors[idx].active_ws_mut().remove(window);
            let Some((_, rect)) = positions.into_iter().find(|(w, _)| *w == window) else {
                return;
            };
            rect
        };
        crate::log_debug!(
            "~pre {} to mon {} ({},{} {}x{}) before it is shown",
            window,
            idx,
            rect.x,
            rect.y,
            rect.width,
            rect.height
        );
        plan.emit(Effect::PrePlace { window, rect });
    }
}
