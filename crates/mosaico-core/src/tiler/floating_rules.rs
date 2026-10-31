//! Moving managed windows between tiled and floating when the rules
//! change.

use crate::WindowId;
use crate::config::RuleAction;

use super::super::removal::untile;
use super::super::{Plan, Tiler};

impl Tiler {
    /// Every managed window the rules float, judged on what the platform
    /// says about it whether or not it is in view.
    pub(in crate::tiler) fn rule_floats(&self, plan: &Plan) -> Vec<WindowId> {
        self.monitors
            .iter()
            .flat_map(|m| &m.workspaces)
            .flat_map(|ws| ws.windows().iter().chain(ws.floating()).copied())
            .filter(|&w| self.rule_holds_floating(plan, w))
            .collect()
    }

    /// Whether the rules float managed `window`, which then floats
    /// whatever the user does: neither the toggle nor a retile tiles it.
    pub(in crate::tiler) fn rule_holds_floating(&self, plan: &Plan, window: WindowId) -> bool {
        plan.facts(window)
            .is_some_and(|facts| self.rules_decide(&facts) == RuleAction::Float)
    }

    /// After the rules changed, floats each managed window they now float
    /// and did not, and tiles each they floated and no longer do. The
    /// others stay as they are, toggled by the user or not. Returns the
    /// monitors to lay out again.
    pub(in crate::tiler) fn follow_rule_floats(
        &mut self,
        plan: &mut Plan,
        floated_before: &[WindowId],
    ) -> Vec<usize> {
        let floats_now = self.rule_floats(plan);
        let tiles_now = floated_before.iter().filter(|w| !floats_now.contains(w));
        let changed: Vec<(WindowId, bool)> = floats_now
            .iter()
            .filter(|w| !floated_before.contains(w))
            .map(|&w| (w, true))
            .chain(tiles_now.map(|&w| (w, false)))
            .collect();
        let mut affected = Vec::new();
        for (window, float) in changed {
            if let Some(mi) = self.follow_rule_change(plan, window, float)
                && !affected.contains(&mi)
            {
                affected.push(mi);
            }
        }
        affected
    }

    /// Floats a managed tile, where it last floated or centered at
    /// two-thirds when its workspace is on screen, or tiles a floating
    /// window at the end of the layout. Returns its monitor, or `None`
    /// when it is held that way already.
    fn follow_rule_change(
        &mut self,
        plan: &mut Plan,
        window: WindowId,
        float: bool,
    ) -> Option<usize> {
        let (mi, wi) = self.find_managed(window)?;
        let on_screen = wi == self.monitors[mi].active_workspace;
        let ws = &mut self.monitors[mi].workspaces[wi];
        if ws.is_floating(window) == float {
            return None;
        }
        if float {
            untile(plan, ws, window);
            ws.end_monocle_of(window);
            ws.add_floating(window);
            if on_screen {
                let rect = self
                    .floats
                    .landing(window, self.monitors[mi].work_area, None);
                self.place_float(plan, window, mi, rect);
                self.float_to_front(plan, window, true);
            }
        } else if on_screen {
            self.unfloat(plan, mi, window);
        } else {
            ws.remove_floating(window);
            ws.add(window);
        }
        crate::log_info!(
            "~rule {} {}",
            window,
            if float { "floats" } else { "tiles" }
        );
        Some(mi)
    }
}
