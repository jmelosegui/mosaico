//! Remembers monitors that disconnect so their windows can go back.
//!
//! When a monitor goes away its windows are moved to the same workspace
//! number on the primary monitor, and each one records where it came
//! from. The monitor's own workspace settings (layout, monocle, active
//! workspace) are parked under its [`MonitorKey`]. When a monitor with
//! that key is connected again, the parked state is restored and every
//! window that is still where it was put goes back to its old workspace
//! and position. A window the user has moved since then stays where the
//! user put it.

use std::collections::HashMap;

use mosaico_core::{MonitorKey, Workspace};

use super::MonitorState;

/// State kept for monitors that are currently disconnected.
#[derive(Default)]
pub(super) struct ReconnectMemory {
    /// Workspace settings of each disconnected monitor, with no windows.
    parked: HashMap<MonitorKey, ParkedMonitor>,
    /// Windows moved off a disconnected monitor, keyed by hwnd.
    displaced: HashMap<usize, Displaced>,
}

struct ParkedMonitor {
    workspaces: Vec<Workspace>,
    active_workspace: usize,
}

/// Where a displaced window lived, and where it was put instead.
struct Displaced {
    home: MonitorKey,
    workspace: usize,
    slot: usize,
    host: MonitorKey,
    host_workspace: usize,
}

impl ReconnectMemory {
    /// Moves every window of a disconnected monitor onto `host` and parks
    /// the monitor's workspace settings. Returns the windows that moved.
    pub(super) fn park(&mut self, mut gone: MonitorState, host: &mut MonitorState) -> Vec<usize> {
        let mut moved = Vec::new();
        for (wi, ws) in gone.workspaces.iter_mut().enumerate() {
            let last_focused = ws.last_focused();
            for (slot, hwnd) in ws.handles().to_vec().into_iter().enumerate() {
                ws.remove(hwnd);
                let target = host.workspaces.len().saturating_sub(1).min(wi);
                host.workspaces[target].add(hwnd);
                moved.push(hwnd);
                self.record(hwnd, &gone.key, wi, slot, &host.key);
            }
            ws.set_last_focused(last_focused);
        }
        if !gone.key.is_unknown() {
            self.parked.insert(
                gone.key,
                ParkedMonitor {
                    workspaces: gone.workspaces,
                    active_workspace: gone.active_workspace,
                },
            );
        }
        moved
    }

    fn record(
        &mut self,
        hwnd: usize,
        home: &MonitorKey,
        ws: usize,
        slot: usize,
        host: &MonitorKey,
    ) {
        if home.is_unknown() || host.is_unknown() {
            // Without both identities the window can never be found
            // again reliably, so it simply stays where it is.
            self.displaced.remove(&hwnd);
            return;
        }
        // A window displaced twice (its host disconnected too) keeps its
        // original home and only follows the new host.
        let entry = self.displaced.entry(hwnd).or_insert_with(|| Displaced {
            home: home.clone(),
            workspace: ws,
            slot,
            host: host.clone(),
            host_workspace: ws,
        });
        entry.host = host.clone();
        entry.host_workspace = ws;
    }

    /// Takes the parked state of a monitor that is connected again.
    pub(super) fn unpark(&mut self, key: &MonitorKey) -> Option<(Vec<Workspace>, usize)> {
        if key.is_unknown() {
            return None;
        }
        self.parked
            .remove(key)
            .map(|p| (p.workspaces, p.active_workspace))
    }

    /// Sends windows whose home is `monitors[home_idx]` back there.
    ///
    /// A window only moves if it is still on the monitor and workspace it
    /// was displaced to; otherwise the user moved it and it stays. Either
    /// way its record is dropped. Returns the windows that moved.
    pub(super) fn restore(&mut self, monitors: &mut [MonitorState], home_idx: usize) -> Vec<usize> {
        let key = monitors[home_idx].key.clone();
        if key.is_unknown() {
            return Vec::new();
        }
        let hwnds: Vec<usize> = self
            .displaced
            .iter()
            .filter(|(_, d)| d.home == key)
            .map(|(&hwnd, _)| hwnd)
            .collect();
        let mut returning: Vec<(usize, Displaced)> = hwnds
            .into_iter()
            .filter_map(|hwnd| self.displaced.remove(&hwnd).map(|d| (hwnd, d)))
            .collect();
        returning.sort_by_key(|(_, d)| (d.workspace, d.slot));

        let mut moved = Vec::new();
        for (hwnd, d) in returning {
            let Some(host) = monitors.iter().position(|m| m.key == d.host) else {
                continue;
            };
            if !monitors[host].workspaces[d.host_workspace].remove(hwnd) {
                continue;
            }
            monitors[home_idx].workspaces[d.workspace].insert(d.slot, hwnd);
            moved.push(hwnd);
        }
        moved
    }

    /// Forgets a window that no longer exists.
    pub(super) fn forget(&mut self, hwnd: usize) {
        self.displaced.remove(&hwnd);
    }

    /// Short summary for `debug state`.
    pub(super) fn describe(&self) -> String {
        let mut parked: Vec<String> = self.parked.keys().map(|k| k.to_string()).collect();
        parked.sort();
        format!(
            "Disconnected monitors: {}, displaced windows: {}",
            if parked.is_empty() {
                "none".into()
            } else {
                parked.join("; ")
            },
            self.displaced.len()
        )
    }
}

#[cfg(test)]
#[path = "tests/reconnect_tests.rs"]
mod tests;
