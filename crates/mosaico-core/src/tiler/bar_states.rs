//! What each monitor's bar shows of the tiler's state.

use crate::bar::BarState;

use super::Tiler;

impl Tiler {
    /// What each monitor's bar shows, in monitor order.
    ///
    /// `update_text` and `paused` come from the daemon and go to every
    /// monitor. Only the monitor the user is working on shows the focused
    /// window.
    pub fn bar_states(&self, update_text: &str, paused: bool) -> Vec<BarState> {
        self.monitors
            .iter()
            .enumerate()
            .map(|(i, mon)| BarState {
                active_workspace: mon.active_workspace,
                workspace_count: mon.workspaces.len(),
                layout: mon.active_ws().layout_kind(),
                monocle: mon.active_ws().monocle(),
                focused_window: if i == self.focused_monitor {
                    self.focused_window
                } else {
                    None
                },
                update_text: update_text.to_string(),
                paused,
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "tests/bar_state_tests.rs"]
mod tests;
