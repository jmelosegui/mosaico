//! Pure monitor-list rebuild for display changes.

use crate::{LayoutKind, Monitor, MonitorKey, match_monitors};

use crate::WindowId;

use super::Display;

use super::MonitorState;
use super::reconnect::ReconnectMemory;

/// Result of [`rebuild_monitors`].
pub(super) struct Rebuilt {
    pub(super) monitors: Vec<MonitorState>,
    /// Windows that changed monitor or workspace, whose visibility has to
    /// be brought in line with their new workspace.
    pub(super) moved: Vec<WindowId>,
    /// For each new monitor, the index of the old monitor it continues.
    pub(super) pairs: Vec<Option<usize>>,
    /// The primary monitor, which took in the windows of the monitors
    /// that went away.
    pub(super) host: usize,
}

/// Builds the monitor list for a new display configuration.
///
/// Each new monitor continues the state of the old monitor it is paired
/// with by [`pair_monitors`]. An unpaired monitor that was connected
/// before gets its parked state back from `memory`, along with the
/// windows that were displaced from it; any other unpaired monitor
/// starts with fresh workspaces using `layouts`. Old monitors without a
/// pair are parked, and their windows move to the same workspace number
/// on the primary monitor.
pub(super) fn rebuild_monitors(
    mut old: Vec<MonitorState>,
    new: &[Display],
    layouts: &[LayoutKind],
    memory: &mut ReconnectMemory,
) -> Rebuilt {
    let pairs = pair_monitors(&old, new);
    let renames: Vec<(MonitorKey, MonitorKey)> = pairs
        .iter()
        .zip(new)
        .filter_map(|(pair, info)| pair.map(|oi| (old[oi].key.clone(), info.key().clone())))
        .collect();
    memory.rehost(&renames);
    let mut returned = Vec::new();

    let mut states: Vec<MonitorState> = new
        .iter()
        .zip(&pairs)
        .enumerate()
        .map(|(ni, (info, pair))| {
            let mut state = MonitorState::new(info, layouts);
            if let Some(oi) = pair {
                let prev = &mut old[*oi];
                state.workspaces = std::mem::take(&mut prev.workspaces);
                state.active_workspace = prev.active_workspace;
            } else if let Some((parked_key, workspaces, active)) = memory.unpark(info.key()) {
                crate::log_info!(
                    "Monitor {} reconnected (parked as {})",
                    info.key(),
                    parked_key
                );
                state.workspaces = workspaces;
                state.active_workspace = active;
                returned.push((ni, parked_key));
            } else {
                crate::log_info!("New monitor {}", info.key());
            }
            state
        })
        .collect();

    let mut moved = Vec::new();
    let host = new.iter().position(|m| m.is_primary()).unwrap_or(0);
    for (oi, gone) in old.into_iter().enumerate() {
        if pairs.contains(&Some(oi)) {
            continue;
        }
        crate::log_info!(
            "Monitor {} disconnected, moving its windows to monitor {}",
            gone.key,
            states[host].key
        );
        moved.extend(memory.park(gone, &mut states[host]));
    }

    for (idx, parked_key) in returned {
        let back = memory.restore(&mut states, idx, &parked_key);
        crate::log_info!(
            "Returned {} window(s) to monitor {}",
            back.len(),
            states[idx].key
        );
        moved.extend(back);
        clear_stale_monocle(&mut states[idx]);
    }

    Rebuilt {
        monitors: states,
        moved,
        pairs,
        host,
    }
}

/// Turns monocle off on workspaces whose monocle window did not return.
fn clear_stale_monocle(monitor: &mut MonitorState) {
    for ws in &mut monitor.workspaces {
        if ws.monocle() && !ws.monocle_window().is_some_and(|h| ws.contains(h)) {
            ws.set_monocle(false);
            ws.set_monocle_window(None);
        }
    }
}

/// Pairs each new monitor with the old monitor it is, if any.
///
/// Uses [`match_monitors`] on the keys. Monitors whose key is unknown
/// (the platform could not identify them) fall back to the same session id
/// and then to the same work area origin, which is the best that can be
/// done without an identity. A monitor with a known key is never paired
/// with another known key by position: that would hand one monitor's
/// workspaces to a different monitor that happens to sit in the same
/// place, such as an office monitor taking over a home one.
pub(super) fn pair_monitors(old: &[MonitorState], new: &[Display]) -> Vec<Option<usize>> {
    let old_keys: Vec<MonitorKey> = old.iter().map(|m| m.key.clone()).collect();
    let new_keys: Vec<MonitorKey> = new.iter().map(|m| m.key().clone()).collect();
    let mut pairs = match_monitors(&old_keys, &new_keys);

    let mut used = vec![false; old.len()];
    for oi in pairs.iter().flatten() {
        used[*oi] = true;
    }

    for (ni, info) in new.iter().enumerate() {
        if pairs[ni].is_some() {
            continue;
        }
        let eligible =
            |oi: usize| !used[oi] && (info.key().is_unknown() || old[oi].key.is_unknown());
        let found = (0..old.len())
            .find(|&oi| eligible(oi) && old[oi].id == info.id)
            .or_else(|| {
                // Compare against the OS work area: the tiling area is
                // shifted down by the bar and would never match.
                (0..old.len()).find(|&oi| {
                    eligible(oi)
                        && old[oi].os_work_area.x == info.work_area().x
                        && old[oi].os_work_area.y == info.work_area().y
                })
            });
        if let Some(oi) = found {
            pairs[ni] = Some(oi);
            used[oi] = true;
        }
    }
    pairs
}
