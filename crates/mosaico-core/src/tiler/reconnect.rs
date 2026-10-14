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

use crate::{MonitorKey, Workspace};

use crate::WindowId;

use super::MonitorState;

#[path = "reconnect_restore.rs"]
mod restore;

/// State kept for monitors that are currently disconnected.
#[derive(Default)]
pub(super) struct ReconnectMemory {
    /// Workspace settings of each disconnected monitor, with no windows.
    parked: HashMap<MonitorKey, ParkedMonitor>,
    /// Windows moved off a disconnected monitor.
    displaced: HashMap<WindowId, Displaced>,
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
    pub(super) fn park(
        &mut self,
        mut gone: MonitorState,
        host: &mut MonitorState,
    ) -> Vec<WindowId> {
        let mut moved = Vec::new();
        for (wi, ws) in gone.workspaces.iter_mut().enumerate() {
            let last_focused = ws.last_focused();
            for (slot, window) in ws.windows().to_vec().into_iter().enumerate() {
                ws.remove(window);
                let target = host.workspaces.len().saturating_sub(1).min(wi);
                host.workspaces[target].add(window);
                moved.push(window);
                self.record(window, &gone.key, wi, slot, &host.key);
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
        window: WindowId,
        home: &MonitorKey,
        ws: usize,
        slot: usize,
        host: &MonitorKey,
    ) {
        if home.is_unknown() {
            // A home that cannot be recognised when it comes back is never
            // parked, so the window simply stays where it is.
            self.displaced.remove(&window);
            return;
        }
        // A window displaced twice (its host disconnected too) keeps its
        // original home and only follows the new host.
        let entry = self.displaced.entry(window).or_insert_with(|| Displaced {
            home: home.clone(),
            workspace: ws,
            slot,
            host: host.clone(),
            host_workspace: ws,
        });
        entry.host = host.clone();
        entry.host_workspace = ws;
    }

    /// Points displaced windows at the new key of their host monitor.
    ///
    /// A monitor that stays connected can still report a different key
    /// after a change, for example a reduced one while a KVM is switched
    /// away and an emulated display stands in for it. `renames` holds
    /// `(old key, new key)` for every monitor paired across the change;
    /// they are applied in one pass so renames cannot chain.
    pub(super) fn rehost(&mut self, renames: &[(MonitorKey, MonitorKey)]) {
        for d in self.displaced.values_mut() {
            if let Some((_, to)) = renames.iter().find(|(from, _)| *from == d.host) {
                d.host = to.clone();
            }
        }
    }

    /// Forgets a window that no longer exists.
    pub(super) fn forget(&mut self, window: WindowId) {
        self.displaced.remove(&window);
    }

    /// Keys of the disconnected monitors, in the order of their printed
    /// form, for `debug state`.
    pub(super) fn disconnected(&self) -> Vec<MonitorKey> {
        let mut keys: Vec<MonitorKey> = self.parked.keys().cloned().collect();
        keys.sort_by_cached_key(ToString::to_string);
        keys
    }

    /// How many windows wait for their monitor to come back.
    pub(super) fn displaced_count(&self) -> usize {
        self.displaced.len()
    }
}

#[cfg(test)]
#[path = "tests/reconnect_tests.rs"]
mod tests;
