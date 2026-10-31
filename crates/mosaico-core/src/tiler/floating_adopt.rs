//! Taking in windows the rules float, and following floating windows the
//! user moves.

use crate::WindowId;
use crate::config::RuleAction;

use super::super::{Plan, Tiler};

impl Tiler {
    /// Whether the rules float `window`, judged on what the platform says
    /// about it now.
    pub(in crate::tiler) fn rules_float(&self, plan: &Plan, window: WindowId) -> bool {
        plan.candidate(window)
            .is_some_and(|facts| self.rules_decide(&facts) == RuleAction::Float)
    }

    /// Takes in a window that floats as it comes in, by the rules or as
    /// it did when it left, onto the workspace on screen of monitor `mi`,
    /// where [`Floats::landing`](super::Floats::landing) says, and focuses
    /// it when `focus`. Returns `false`, doing nothing, when a workspace
    /// already holds it.
    pub(in crate::tiler) fn adopt_floating(
        &mut self,
        plan: &mut Plan,
        window: WindowId,
        mi: usize,
        focus: bool,
    ) -> bool {
        if self.monitors.get(mi).is_none() || self.find_managed(window).is_some() {
            return false;
        }
        self.floats.came_back(window);
        self.float_on(plan, mi, window);
        self.learn_min_size(plan, window);
        let area = self.monitors[mi].work_area;
        let rect = self.floats.landing(window, area, plan.rect(window));
        crate::log_info!("+float {} on mon {}", window, mi);
        self.decorate(plan, window);
        self.place_float(plan, window, mi, rect);
        if focus {
            self.focused_monitor = mi;
            self.focus_window(plan, window, false);
        }
        true
    }

    /// Floats `window` on the workspace on screen of monitor `mi`, in
    /// front of the others there, ending monocle there to show it.
    pub(in crate::tiler) fn float_on(&mut self, plan: &mut Plan, mi: usize, window: WindowId) {
        if self.monitors[mi].active_ws().monocle() {
            self.exit_monocle(plan, mi);
        }
        self.monitors[mi].active_ws_mut().add_floating(window);
    }

    /// Follows a floating window the user moved, as by dragging it: one
    /// that now is on another monitor joins the workspace on screen there,
    /// where they put it. It is not moved, only its border follows.
    pub(in crate::tiler) fn float_moved(&mut self, plan: &mut Plan, window: WindowId) {
        let Some((from, wi)) = self.find_floating(window) else {
            return;
        };
        let to = self.monitor_index_for(plan, window);
        if let Some(to) = to.filter(|&to| to != from)
            && wi == self.monitors[from].active_workspace
        {
            self.monitors[from].workspaces[wi].remove_floating(window);
            self.float_on(plan, to, window);
            if self.focused_window == Some(window) {
                self.focused_monitor = to;
            }
            crate::log_info!("float {} dropped on mon {}", window, to);
        }
        self.update_border(plan);
    }
}
