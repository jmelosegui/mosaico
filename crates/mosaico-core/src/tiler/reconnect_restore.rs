//! Bringing back the state and the windows of a monitor that is
//! connected again.

use crate::{LayoutKind, MonitorKey, Workspace, match_monitors};

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
    /// each with the split of the BSP chain it had there, once all are back
    /// in their places.
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
            if !monitors[host].workspaces[d.host_workspace].remove(window) {
                continue;
            }
            monitors[home_idx].workspaces[d.workspace].insert(d.slot, window);
            moved.push(window);
            splits.extend(d.split.map(|ratio| (window, d.workspace, ratio)));
        }
        for (window, wi, ratio) in splits {
            let ws = &mut monitors[home_idx].workspaces[wi];
            if let Some(split) = ws.index_of(window) {
                ws.set_split_ratio_of(LayoutKind::Bsp, split, ratio);
            }
        }
        moved
    }
}

#[cfg(test)]
#[path = "tests/reconnect_split_tests.rs"]
mod split_tests;

#[cfg(test)]
#[path = "tests/reconnect_away_tests.rs"]
mod away_tests;
