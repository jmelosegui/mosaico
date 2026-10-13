use super::*;

#[cfg(test)]
#[path = "tests/display_tests.rs"]
mod display_tests;
#[cfg(test)]
#[path = "tests/empty_focus_tests.rs"]
mod empty_focus_tests;
#[cfg(test)]
#[path = "tests/hiding_tests.rs"]
mod hiding_tests;
#[cfg(test)]
#[path = "tests/minimize_tests.rs"]
mod minimize_tests;
#[cfg(test)]
#[path = "tests/monitor_tests.rs"]
mod monitor_tests;
#[cfg(test)]
#[path = "tests/monocle_tests.rs"]
mod monocle_tests;
#[cfg(test)]
#[path = "tests/pre_placed_tests.rs"]
mod pre_placed_tests;
#[cfg(test)]
#[path = "tests/work_area_tests.rs"]
mod work_area_tests;
#[cfg(test)]
#[path = "tests/workspace_tests.rs"]
mod workspace_tests;

pub(super) fn make_monitor(id: usize) -> MonitorState {
    MonitorState {
        id,
        key: MonitorKey::default(),
        os_work_area: Rect::new(0, 0, 1920, 1080),
        work_area: Rect::new(0, 0, 1920, 1080),
        workspaces: (0..MAX_WORKSPACES).map(|_| Workspace::new()).collect(),
        active_workspace: 0,
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

/// Runs the real display change rebuild with unidentified monitors, the
/// first of which is primary.
pub(super) fn simulate_display_change(
    old_monitors: &mut Vec<MonitorState>,
    new_infos: Vec<(usize, Rect)>,
) -> Vec<MonitorState> {
    let infos: Vec<crate::monitor::MonitorInfo> = new_infos
        .into_iter()
        .enumerate()
        .map(|(i, (id, work_area))| crate::monitor::MonitorInfo {
            id,
            key: MonitorKey::default(),
            work_area,
            primary: i == 0,
        })
        .collect();
    let layouts = vec![LayoutKind::default(); MAX_WORKSPACES as usize];
    display_rebuild::rebuild_monitors(std::mem::take(old_monitors), &infos, &layouts)
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
