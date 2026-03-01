use super::*;

#[cfg(test)]
#[path = "tests/display_tests.rs"]
mod display_tests;
#[cfg(test)]
#[path = "tests/hiding_tests.rs"]
mod hiding_tests;
#[cfg(test)]
#[path = "tests/monitor_tests.rs"]
mod monitor_tests;
#[cfg(test)]
#[path = "tests/monocle_tests.rs"]
mod monocle_tests;
#[cfg(test)]
#[path = "tests/workspace_tests.rs"]
mod workspace_tests;

pub(super) fn make_monitor(id: usize) -> MonitorState {
    MonitorState {
        id,
        work_area: Rect::new(0, 0, 1920, 1080),
        workspaces: (0..MAX_WORKSPACES).map(|_| Workspace::new()).collect(),
        active_workspace: 0,
        monocle: false,
        monocle_window: None,
    }
}

// -- MonitorState helpers --

#[test]
fn active_ws_returns_correct_workspace() {
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(100);
    mon.workspaces[1].add(200);

    assert!(mon.active_ws().contains(100));
    assert!(!mon.active_ws().contains(200));

    mon.active_workspace = 1;
    assert!(!mon.active_ws().contains(100));
    assert!(mon.active_ws().contains(200));
}

#[test]
fn active_ws_mut_modifies_correct_workspace() {
    let mut mon = make_monitor(1);
    mon.active_ws_mut().add(100);
    assert!(mon.workspaces[0].contains(100));

    mon.active_workspace = 1;
    mon.active_ws_mut().add(200);
    assert!(mon.workspaces[1].contains(200));
    assert!(!mon.workspaces[1].contains(100));
}

#[test]
fn monitor_state_has_max_workspaces() {
    let mon = make_monitor(1);
    assert_eq!(mon.workspaces.len(), MAX_WORKSPACES as usize);
}

// -- find_window --

pub(super) fn make_monitors(n: usize) -> Vec<MonitorState> {
    (0..n).map(make_monitor).collect()
}

// -- hidden_by_switch --

#[test]
fn hidden_by_switch_tracks_hwnds() {
    let mut set = HashSet::new();

    // Simulate hiding windows for workspace switch
    set.insert(100);
    set.insert(200);
    assert!(set.contains(&100));
    assert!(set.contains(&200));
    assert!(!set.contains(&300));

    // Simulate showing them again
    set.remove(&100);
    assert!(!set.contains(&100));
    assert!(set.contains(&200));
}

#[test]
fn hidden_by_switch_ignores_duplicates() {
    let mut set = HashSet::new();
    set.insert(100);
    set.insert(100); // no-op
    assert_eq!(set.len(), 1);
}

// -- workspace switch simulation --

pub(super) fn simulate_display_change(
    old_monitors: &mut [MonitorState],
    new_infos: Vec<(usize, Rect)>,
) -> Vec<MonitorState> {
    let mut new_states: Vec<MonitorState> = Vec::new();

    for (id, work_area) in &new_infos {
        let old_idx = old_monitors.iter().position(|m| m.id == *id).or_else(|| {
            old_monitors
                .iter()
                .position(|m| m.work_area.x == work_area.x && m.work_area.y == work_area.y)
        });

        if let Some(idx) = old_idx {
            let old = &mut old_monitors[idx];
            new_states.push(MonitorState {
                id: *id,
                work_area: *work_area,
                workspaces: std::mem::take(&mut old.workspaces),
                active_workspace: old.active_workspace,
                monocle: old.monocle,
                monocle_window: old.monocle_window,
            });
        } else {
            new_states.push(MonitorState {
                id: *id,
                work_area: *work_area,
                workspaces: (0..MAX_WORKSPACES).map(|_| Workspace::new()).collect(),
                active_workspace: 0,
                monocle: false,
                monocle_window: None,
            });
        }
    }

    // Migrate windows from removed monitors.
    for old_mon in old_monitors.iter() {
        if old_mon.workspaces.is_empty() {
            continue;
        }
        for ws in &old_mon.workspaces {
            for &hwnd in ws.handles() {
                new_states[0].active_ws_mut().add(hwnd);
            }
        }
    }

    new_states
}

// Standalone helper matching TilingManager::find_window logic
pub(super) fn find_window_in(monitors: &[MonitorState], hwnd: usize) -> Option<(usize, usize)> {
    for (mi, mon) in monitors.iter().enumerate() {
        for (wi, ws) in mon.workspaces.iter().enumerate() {
            if ws.contains(hwnd) {
                return Some((mi, wi));
            }
        }
    }
    None
}
