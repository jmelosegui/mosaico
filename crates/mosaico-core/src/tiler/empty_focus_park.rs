//! Parking the focused monitor when the user lands on an empty
//! workspace, and the focus events that may move it while it is
//! parked. The claim and the judgement are in `empty_focus`.

use super::empty_focus::{ClaimVerdict, EmptyFocusClaim, judge_focus_event, monitor_at_point};
use super::{Plan, Tiler};

impl Tiler {
    /// Parks `focused_monitor` when the workspace the user just landed
    /// on is empty, so focus restores elsewhere cannot move it.
    ///
    /// Call this at the end of a navigation, once the new active
    /// workspace is in place.
    pub(super) fn park_focus_if_empty(&mut self, plan: &Plan) {
        let idx = self.focused_monitor;
        let empty = self
            .monitors
            .get(idx)
            .is_some_and(|mon| mon.active_ws().is_empty());

        if !empty {
            // Landing somewhere with windows resolves any earlier claim.
            self.empty_focus_claim = None;
            return;
        }

        crate::log_debug!("focus-parked mon {} (workspace is empty)", idx);
        self.empty_focus_claim = Some(EmptyFocusClaim::new(idx, plan.now()));
    }

    /// Returns whether a focus event may move `focused_monitor`,
    /// consuming the claim when the event resolves it.
    pub(super) fn focus_event_allowed(&mut self, plan: &Plan, event_monitor: usize) -> bool {
        let Some(claim) = self.empty_focus_claim else {
            return true;
        };

        let cursor = plan
            .cursor()
            .and_then(|(x, y)| monitor_at_point(&self.monitors, x, y));
        match judge_focus_event(&self.monitors, &claim, event_monitor, cursor, plan.now()) {
            ClaimVerdict::Apply => {
                self.empty_focus_claim = None;
                true
            }
            ClaimVerdict::Ignore => false,
        }
    }
}
