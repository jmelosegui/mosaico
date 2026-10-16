//! Monitors connecting and disconnecting.

use std::collections::HashSet;

use crate::config::WorkspaceMode;

use crate::WindowId;

use super::display_monocle::join_monocle;
use super::display_rebuild::rebuild_monitors;
use super::{BarReservation, Display, Effect, Plan, Tiler};

impl Tiler {
    /// Rebuilds the monitor state after the set of monitors changed.
    ///
    /// Monitors still connected keep their workspaces. They are
    /// recognized by [`crate::MonitorKey`], since platforms may hand out
    /// new session ids on every change, including for monitors that
    /// never went away. Windows on removed monitors move to the same
    /// workspace number on the primary monitor, and go back when the
    /// monitor reconnects (see [`super::reconnect`]).
    pub(super) fn handle_display_change(
        &mut self,
        plan: &mut Plan,
        displays: Vec<Display>,
        bar: &BarReservation,
    ) {
        if displays.is_empty() {
            return;
        }

        // Several change notifications can describe the same layout. The
        // bars may still have changed with it, when one could not be made
        // again or was made for the first time, so their space follows.
        let unchanged = displays.len() == self.monitors.len()
            && displays
                .iter()
                .zip(&self.monitors)
                .all(|(new, old)| new.id == old.id && new.work_area == old.os_work_area);
        if unchanged {
            self.reserve_bar_if_moved(plan, bar);
            return;
        }

        crate::log_info!(
            "Display change: {} -> {} monitors",
            self.monitors.len(),
            displays.len()
        );
        // What a platform kept of a border may be lost with the change, so
        // every border is drawn from scratch.
        plan.emit(Effect::RedrawBorders);

        let focused = self.focused_monitor;
        let in_view = self.monitors.get(focused).map(|m| m.active_workspace);
        let old = std::mem::take(&mut self.monitors);
        let rebuilt =
            rebuild_monitors(old, &displays, &self.workspace_layouts, &mut self.reconnect);
        self.monitors = rebuilt.monitors;

        // Monitor indices were just rebuilt, so a parked focus claim may
        // refer to a different monitor now.
        self.empty_focus_claim = None;

        // Stay on the same physical monitor if it is still connected,
        // followed through the pairing, since its key can change while it
        // stays connected. Otherwise follow its windows to the primary
        // monitor, which took them in (#129).
        self.focused_monitor = rebuilt
            .pairs
            .iter()
            .position(|&old| old == Some(focused))
            .unwrap_or(rebuilt.host);

        // In global mode the monitors are always in step: every switch
        // moves all of them, and a reload that turns the mode on lines them
        // up (#128). The workspace in view before the change stays in view,
        // and monitors that connect join it, even when the monitor that had
        // focus is the one that went away (#124). Only a returning monitor
        // should switch here, and its windows moved already. A monitor out
        // of step anyway has its windows hidden and shown below, with the
        // windows that moved, as a safeguard.
        let mut moved = rebuilt.moved;
        let from_elsewhere = moved.clone();
        if self.workspace_mode == WorkspaceMode::Global
            && let Some(target) = in_view
        {
            for (mon_idx, mon) in self.monitors.iter_mut().enumerate() {
                let windows = mon.switch_to(target);
                if windows.iter().any(|w| !moved.contains(w)) {
                    crate::log_warn!(
                        "Mon {} was out of step in global mode, switched it to ws {}",
                        mon_idx,
                        target + 1
                    );
                }
                moved.extend(windows);
            }
        }
        // A window can be listed twice: parked off a monitor that left and
        // sent back home to one that returned in the same change, or also
        // on a workspace a monitor switched. Each is hidden or shown once.
        let mut seen = HashSet::new();
        moved.retain(|&window| seen.insert(window));
        // Windows from another monitor wait behind the monocle window of a
        // monocle workspace they land on, and one that brings focus hands
        // it to the monocle window as focus settles (#132). Only the
        // windows the rebuild moved count, once the line-up has settled
        // which workspaces are in view.
        let waiting = join_monocle(
            &mut self.monitors,
            &from_elsewhere,
            &rebuilt.went_home,
            self.focused_window,
        );

        // Focus settles first, so a focused window about to be hidden has
        // handed it on and the system's own pick is not followed. Windows
        // that landed on an inactive workspace are hidden before the
        // retile; the ones on an active workspace are shown after it, so
        // they appear in their final place. One whose slot cannot be set
        // while it is out of view is shown before it instead, as a
        // workspace switch does (see `HiddenHow::shown_first`).
        let (shown, hidden): (Vec<WindowId>, Vec<WindowId>) = moved
            .into_iter()
            .partition(|&w| self.is_on_active_workspace(w));
        self.settle_focus(plan, &hidden, &waiting);
        let how = self.layout.hiding;
        for &window in &hidden {
            self.hidden_how.hide(plan, window, how);
        }
        let (restored, shown): (Vec<WindowId>, Vec<WindowId>) = shown
            .into_iter()
            .partition(|&w| self.hidden_how.shown_first(plan, w));
        for &window in &restored {
            self.hidden_how.show(plan, window, how);
        }

        self.reserve_bar(plan, bar);

        for &window in &shown {
            self.hidden_how.show(plan, window, how);
        }
        self.put_shown_behind(plan, &waiting, &shown);
        self.repaint_visible(plan);
        self.update_border(plan);
    }
}

#[cfg(test)]
#[path = "tests/display_bar_tests.rs"]
mod tests;
