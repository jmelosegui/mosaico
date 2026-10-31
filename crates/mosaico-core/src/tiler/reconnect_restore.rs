//! Bringing back the state and the windows of a monitor that is
//! connected again.

use crate::layout::OwnSplits;
use crate::{MonitorKey, Workspace, match_monitors};

use crate::WindowId;

use super::super::MonitorState;
use super::{Displaced, ReconnectMemory};

impl ReconnectMemory {
    /// Takes the parked state of a monitor that is connected again.
    ///
    /// The parked key is found with [`match_monitors`], not by equality,
    /// so a monitor that comes back reporting less than it did (no serial
    /// or device path yet) still finds its state. Returns the key it was
    /// parked under, which is what its displaced windows refer to.
    pub(crate) fn unpark(
        &mut self,
        key: &MonitorKey,
    ) -> Option<(MonitorKey, Vec<Workspace>, usize)> {
        let parked: Vec<MonitorKey> = self.parked.keys().cloned().collect();
        let found = match_monitors(&parked, std::slice::from_ref(key))[0]?;
        let parked_key = parked[found].clone();
        self.parked
            .remove(&parked_key)
            .map(|p| (parked_key, p.workspaces, p.active_workspace))
    }

    /// Sends windows whose home is `home` back to `monitors[home_idx]`,
    /// each with the splits and the share of its stacks it had there, once
    /// all are back in their places: in every layout, those it still has
    /// where it is, so a whole stack back has its heights back.
    ///
    /// A window only moves if it is still on the monitor and workspace it
    /// was displaced to; otherwise the user moved it and it stays. Either
    /// way its record is dropped. Returns the windows that moved.
    pub(crate) fn restore(
        &mut self,
        monitors: &mut [MonitorState],
        home_idx: usize,
        home: &MonitorKey,
    ) -> Vec<WindowId> {
        let windows: Vec<WindowId> = self
            .displaced
            .iter()
            .filter(|(_, d)| d.home == *home)
            .map(|(&window, _)| window)
            .collect();
        let mut returning: Vec<(WindowId, Displaced)> = windows
            .into_iter()
            .filter_map(|window| self.displaced.remove(&window).map(|d| (window, d)))
            .collect();
        returning.sort_by_key(|(_, d)| (d.workspace, d.slot));

        let mut moved = Vec::new();
        let mut splits = Vec::new();
        for (window, d) in returning {
            let Some(host) = monitors.iter().position(|m| m.key == d.host) else {
                continue;
            };
            if d.floating {
                if monitors[host].workspaces[d.host_workspace].remove_floating(window) {
                    monitors[home_idx].workspaces[d.workspace].add_floating(window);
                    moved.push(window);
                }
                continue;
            }
            if !monitors[host].workspaces[d.host_workspace].remove(window) {
                continue;
            }
            monitors[home_idx].workspaces[d.workspace].insert(d.slot, window);
            moved.push(window);
            splits.push((window, d.workspace, d.slot, d.splits));
        }
        for (wi, ws) in monitors[home_idx].workspaces.iter_mut().enumerate() {
            let here = splits.iter().filter(|back| back.1 == wi);
            let at = |&(window, _, slot, own): &(WindowId, usize, usize, OwnSplits)| {
                Some((ws.index_of(window)?, slot, own))
            };
            let back: Vec<_> = here.filter_map(at).collect();
            if !back.is_empty() {
                ws.give_splits(&back);
            }
        }
        moved
    }

    /// Moves the floating windows of `ws`, workspace `wi` of the monitor
    /// `home` that went away, to workspace `target` of `host`, where they
    /// float too. Returns them.
    pub(super) fn park_floats(
        &mut self,
        ws: &mut Workspace,
        (wi, target): (usize, usize),
        home: &MonitorKey,
        host: &mut MonitorState,
    ) -> Vec<WindowId> {
        let floating = ws.floating().to_vec();
        for &window in &floating {
            ws.remove_floating(window);
            host.workspaces[target].add_floating(window);
            self.record(window, home, (wi, 0, OwnSplits::default()), &host.key);
            if let Some(d) = self.displaced.get_mut(&window) {
                d.floating = true;
            }
        }
        floating
    }
}

#[cfg(test)]
#[path = "tests/reconnect_split_tests.rs"]
mod split_tests;

#[cfg(test)]
#[path = "tests/reconnect_away_tests.rs"]
mod away_tests;

#[cfg(test)]
#[path = "tests/reconnect_stack_tests.rs"]
mod stack_tests;
