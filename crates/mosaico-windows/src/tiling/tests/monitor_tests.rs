use super::super::*;
use super::{find_window_in, make_monitor, make_monitors};

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

#[test]
fn find_window_on_active_workspace() {
    let mut monitors = make_monitors(2);
    monitors[0].workspaces[0].add(100);
    monitors[1].workspaces[0].add(200);

    // find_window is a method on TilingManager; test the logic directly
    let result = find_window_in(&monitors, 100);
    assert_eq!(result, Some((0, 0)));

    let result = find_window_in(&monitors, 200);
    assert_eq!(result, Some((1, 0)));
}

#[test]
fn find_window_on_non_active_workspace() {
    let mut monitors = make_monitors(1);
    monitors[0].workspaces[3].add(42);
    monitors[0].active_workspace = 0;

    let result = find_window_in(&monitors, 42);
    assert_eq!(result, Some((0, 3)));
}

#[test]
fn find_window_not_found() {
    let monitors = make_monitors(2);
    assert_eq!(find_window_in(&monitors, 999), None);
}

#[test]
fn owning_monitor_searches_all_workspaces() {
    let mut monitors = make_monitors(2);
    monitors[1].workspaces[5].add(77);

    let result = monitors
        .iter()
        .position(|m| m.workspaces.iter().any(|ws| ws.contains(77)));
    assert_eq!(result, Some(1));
}
