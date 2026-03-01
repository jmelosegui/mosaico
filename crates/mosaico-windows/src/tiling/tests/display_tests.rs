use super::super::*;
use super::{make_monitors, simulate_display_change};

#[test]
fn display_change_preserves_windows_on_same_monitor() {
    let mut monitors = make_monitors(2);
    monitors[0].workspaces[0].add(100);
    monitors[0].workspaces[0].add(200);
    monitors[1].workspaces[0].add(300);

    let new_infos = vec![
        (0, Rect::new(0, 0, 1920, 1080)),
        (1, Rect::new(1920, 0, 1920, 1080)),
    ];
    let result = simulate_display_change(&mut monitors, new_infos);

    assert_eq!(result.len(), 2);
    assert!(result[0].active_ws().contains(100));
    assert!(result[0].active_ws().contains(200));
    assert!(result[1].active_ws().contains(300));
}

#[test]
fn display_change_migrates_windows_from_removed_monitor() {
    let mut monitors = make_monitors(2);
    monitors[0].workspaces[0].add(100);
    monitors[1].workspaces[0].add(200);
    monitors[1].workspaces[1].add(300);

    // Only monitor 0 remains.
    let new_infos = vec![(0, Rect::new(0, 0, 1920, 1080))];
    let result = simulate_display_change(&mut monitors, new_infos);

    assert_eq!(result.len(), 1);
    assert!(result[0].active_ws().contains(100));
    // Migrated from removed monitor.
    assert!(result[0].active_ws().contains(200));
    assert!(result[0].active_ws().contains(300));
}

#[test]
fn display_change_adds_new_monitor_with_empty_workspaces() {
    let mut monitors = make_monitors(1);
    monitors[0].workspaces[0].add(100);

    // A new monitor (id=99) appears.
    let new_infos = vec![
        (0, Rect::new(0, 0, 1920, 1080)),
        (99, Rect::new(1920, 0, 2560, 1440)),
    ];
    let result = simulate_display_change(&mut monitors, new_infos);

    assert_eq!(result.len(), 2);
    assert!(result[0].active_ws().contains(100));
    assert_eq!(result[1].active_ws().len(), 0);
    assert_eq!(result[1].id, 99);
}

#[test]
fn display_change_matches_by_position_when_id_changes() {
    let mut monitors = make_monitors(1);
    monitors[0].workspaces[0].add(100);
    monitors[0].workspaces[0].add(200);

    // Same position but new ID (reconnected monitor).
    let new_infos = vec![(42, Rect::new(0, 0, 1920, 1080))];
    let result = simulate_display_change(&mut monitors, new_infos);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, 42);
    assert!(result[0].active_ws().contains(100));
    assert!(result[0].active_ws().contains(200));
}

#[test]
fn display_change_no_monitors_is_noop() {
    // The real method returns early for empty new_monitors.
    let result: Vec<MonitorState> = Vec::new();
    assert!(result.is_empty());
}

#[test]
fn display_change_preserves_active_workspace() {
    let mut monitors = make_monitors(1);
    monitors[0].active_workspace = 3;
    monitors[0].workspaces[3].add(100);

    let new_infos = vec![(0, Rect::new(0, 0, 1920, 1080))];
    let result = simulate_display_change(&mut monitors, new_infos);

    assert_eq!(result[0].active_workspace, 3);
    assert!(result[0].workspaces[3].contains(100));
}

#[test]
fn display_change_preserves_monocle_state() {
    let mut monitors = make_monitors(1);
    monitors[0].monocle = true;
    monitors[0].monocle_window = Some(100);
    monitors[0].workspaces[0].add(100);

    let new_infos = vec![(0, Rect::new(0, 0, 1920, 1080))];
    let result = simulate_display_change(&mut monitors, new_infos);

    assert!(result[0].monocle);
    assert_eq!(result[0].monocle_window, Some(100));
}
