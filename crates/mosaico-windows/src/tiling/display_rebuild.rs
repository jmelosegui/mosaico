//! Pure monitor-list rebuild for display changes, kept free of Win32
//! calls so it can be unit tested.

use mosaico_core::{LayoutKind, MonitorKey, match_monitors};

use crate::monitor::MonitorInfo;

use super::MonitorState;

/// Builds the monitor list for a new display configuration.
///
/// Each new monitor continues the state of the old monitor it is paired
/// with by [`pair_monitors`], or starts with fresh workspaces using
/// `layouts`. Windows on old monitors that have no pair are moved to the
/// active workspace of the primary monitor.
pub(super) fn rebuild_monitors(
    mut old: Vec<MonitorState>,
    new: &[MonitorInfo],
    layouts: &[LayoutKind],
) -> Vec<MonitorState> {
    let pairs = pair_monitors(&old, new);

    let mut states: Vec<MonitorState> = new
        .iter()
        .zip(&pairs)
        .map(|(info, pair)| match pair {
            Some(oi) => {
                let prev = &mut old[*oi];
                MonitorState {
                    id: info.id,
                    key: info.key.clone(),
                    os_work_area: info.work_area,
                    work_area: info.work_area,
                    workspaces: std::mem::take(&mut prev.workspaces),
                    active_workspace: prev.active_workspace,
                }
            }
            None => {
                mosaico_core::log_info!("New monitor {}", info.key);
                MonitorState::new(info, layouts)
            }
        })
        .collect();

    let fallback = new.iter().position(|m| m.primary).unwrap_or(0);
    for (oi, prev) in old.iter().enumerate() {
        if pairs.contains(&Some(oi)) {
            continue;
        }
        mosaico_core::log_info!("Monitor {} disconnected", prev.key);
        for ws in &prev.workspaces {
            for &hwnd in ws.handles() {
                mosaico_core::log_info!(
                    "Migrating window 0x{:X} from removed monitor {} to monitor {}",
                    hwnd,
                    prev.id,
                    states[fallback].id
                );
                states[fallback].active_ws_mut().add(hwnd);
            }
        }
    }

    states
}

/// Pairs each new monitor with the old monitor it is, if any.
///
/// Uses [`match_monitors`] on the keys. Monitors whose key is unknown
/// (the platform could not identify them) fall back to the same HMONITOR
/// and then to the same work area origin, which is the best that can be
/// done without an identity. A monitor with a known key is never paired
/// with another known key by position: that would hand one monitor's
/// workspaces to a different monitor that happens to sit in the same
/// place, such as an office monitor taking over a home one.
pub(super) fn pair_monitors(old: &[MonitorState], new: &[MonitorInfo]) -> Vec<Option<usize>> {
    let old_keys: Vec<MonitorKey> = old.iter().map(|m| m.key.clone()).collect();
    let new_keys: Vec<MonitorKey> = new.iter().map(|m| m.key.clone()).collect();
    let mut pairs = match_monitors(&old_keys, &new_keys);

    let mut used = vec![false; old.len()];
    for oi in pairs.iter().flatten() {
        used[*oi] = true;
    }

    for (ni, info) in new.iter().enumerate() {
        if pairs[ni].is_some() {
            continue;
        }
        let eligible = |oi: usize| !used[oi] && (info.key.is_unknown() || old[oi].key.is_unknown());
        let found = (0..old.len())
            .find(|&oi| eligible(oi) && old[oi].id == info.id)
            .or_else(|| {
                // Compare against the OS work area: the tiling area is
                // shifted down by the bar and would never match.
                (0..old.len()).find(|&oi| {
                    eligible(oi)
                        && old[oi].os_work_area.x == info.work_area.x
                        && old[oi].os_work_area.y == info.work_area.y
                })
            });
        if let Some(oi) = found {
            pairs[ni] = Some(oi);
            used[oi] = true;
        }
    }
    pairs
}
