//! Deciding which windows are managed, and taking them in.

use crate::config;

use crate::WindowId;

use super::{Plan, Tiler, WindowFacts};

impl Tiler {
    pub(super) fn is_tileable(&self, plan: &Plan, window: WindowId) -> bool {
        plan.is_visible(window) && self.passes_tiling_rules(plan, window)
    }

    /// Everything `is_tileable` checks except visibility, so a window
    /// that has been created but not shown yet can be judged.
    pub(super) fn passes_tiling_rules(&self, plan: &Plan, window: WindowId) -> bool {
        plan.candidate(window)
            .is_some_and(|facts| self.rules_manage(&facts))
    }

    /// Whether the rules manage a window with these facts.
    pub(super) fn rules_manage(&self, facts: &WindowFacts) -> bool {
        if facts.has_caption {
            config::should_manage(&facts.class, &facts.title, &facts.path, &self.rules)
        } else {
            // No title bar: only tile when a rule explicitly asks for it.
            config::matching_rule(&facts.class, &facts.title, &facts.path, &self.rules)
                == Some(true)
        }
    }

    /// Moves a window to the workspace of the monitor it is now on,
    /// after the user dragged it, and retiles. A window on a workspace out
    /// of view, as one whose workspace the user switched away from while
    /// they dragged it, stays where it is.
    pub(super) fn reassign_monitor(&mut self, plan: &mut Plan, window: WindowId) {
        let old = self.owning_monitor(window);
        if old.is_some() && !self.is_on_active_workspace(window) {
            return;
        }
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
                plan.take_in(window);
                self.move_before_layout(plan, to, window);
                self.apply_layout_on(plan, to);
            }
            _ => {}
        }
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
        plan.take_in(window);
        self.move_before_layout(plan, idx, window);
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
        // Focus before the layout so monocle sizes the right window,
        // and make the newest window the monocle target.
        self.focused_window = Some(window);
        if self.monitors[idx].active_ws().monocle() {
            self.monitors[idx]
                .active_ws_mut()
                .set_monocle_window(Some(window));
        }
        plan.take_in(window);
        self.move_before_layout(plan, idx, window);
        self.decorate(plan, window);
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
}
