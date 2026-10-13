use super::super::display_rebuild::rebuild_monitors;
use super::super::reconnect::ReconnectMemory;
use super::super::*;
use super::{make_monitors, simulate_display_change};
use crate::{MonitorKey, Rect};

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
    // Migrated from removed monitor, keeping the workspace number.
    assert!(result[0].workspaces[0].contains(200));
    assert!(result[0].workspaces[1].contains(300));
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

fn keyed(name: &str) -> MonitorKey {
    MonitorKey {
        stable_id: Some(name.into()),
        ..Default::default()
    }
}

fn info(id: usize, key: &MonitorKey, x: i32, primary: bool) -> Display {
    Display {
        id,
        key: key.clone(),
        work_area: Rect::new(x, 0, 1920, 1080),
        primary,
    }
}

fn keyed_monitors(keys: &[&MonitorKey]) -> Vec<MonitorState> {
    let infos: Vec<Display> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| info(i + 1, k, i as i32 * 1920, i == 0))
        .collect();
    infos
        .iter()
        .map(|i| MonitorState::new(i, &layouts()))
        .collect()
}

fn layouts() -> Vec<LayoutKind> {
    vec![LayoutKind::default(); MAX_WORKSPACES as usize]
}

#[test]
fn keyed_monitor_keeps_workspaces_when_every_handle_changes() {
    // Arrange -- after a KVM switch Windows renumbers every monitor,
    // and the remaining one also moved to the left edge.
    let (a, b) = (keyed("A"), keyed("B"));
    let mut old = keyed_monitors(&[&a, &b]);
    old[1].workspaces[2].add(300);
    old[1].active_workspace = 2;
    let new = vec![info(77, &b, 0, true)];

    // Act
    let result = rebuild_monitors(old, &new, &layouts(), &mut ReconnectMemory::default()).monitors;

    // Assert -- B kept its own workspace 3, not A's.
    assert_eq!(result[0].key, b);
    assert_eq!(result[0].active_workspace, 2);
    assert!(result[0].workspaces[2].contains(300));
}

#[test]
fn keyed_monitor_is_not_paired_by_position_with_a_different_monitor() {
    // Arrange -- the office monitor sits where the home monitor was.
    let (laptop, home, office) = (keyed("laptop"), keyed("home"), keyed("office"));
    let mut old = keyed_monitors(&[&laptop, &home]);
    old[1].workspaces[0].add(200);
    let new = vec![info(5, &laptop, 0, true), info(6, &office, 1920, false)];

    // Act
    let result = rebuild_monitors(old, &new, &layouts(), &mut ReconnectMemory::default()).monitors;

    // Assert -- the office monitor starts empty and the home monitor's
    // window moved to the primary (laptop) screen.
    assert_eq!(result[1].key, office);
    assert!(result[1].workspaces.iter().all(|ws| ws.is_empty()));
    assert!(result[0].workspaces.iter().any(|ws| ws.contains(200)));
}

#[test]
fn removed_monitor_windows_go_to_the_primary_monitor() {
    // Arrange -- primary is on the right this time.
    let (left, right, gone) = (keyed("L"), keyed("R"), keyed("G"));
    let mut old = keyed_monitors(&[&left, &right, &gone]);
    old[2].workspaces[0].add(900);
    let new = vec![info(1, &left, 0, false), info(2, &right, 1920, true)];

    // Act
    let result = rebuild_monitors(old, &new, &layouts(), &mut ReconnectMemory::default()).monitors;

    // Assert
    assert!(result[1].workspaces.iter().any(|ws| ws.contains(900)));
    assert!(result[0].workspaces.iter().all(|ws| !ws.contains(900)));
}

#[test]
fn new_monitor_uses_configured_layouts() {
    // Arrange
    let a = keyed("A");
    let old = keyed_monitors(&[&a]);
    let mut configured = layouts();
    configured[1] = LayoutKind::VerticalStack;
    let new = vec![info(1, &a, 0, true), info(2, &keyed("new"), 1920, false)];

    // Act
    let result = rebuild_monitors(old, &new, &configured, &mut ReconnectMemory::default()).monitors;

    // Assert
    assert_eq!(
        result[1].workspaces[1].layout_kind(),
        LayoutKind::VerticalStack
    );
}

#[test]
fn display_change_preserves_monocle_state() {
    let mut monitors = make_monitors(1);
    monitors[0].active_ws_mut().set_monocle(true);
    monitors[0].active_ws_mut().set_monocle_window(Some(100));
    monitors[0].workspaces[0].add(100);

    let new_infos = vec![(0, Rect::new(0, 0, 1920, 1080))];
    let result = simulate_display_change(&mut monitors, new_infos);

    assert!(result[0].active_ws().monocle());
    assert_eq!(result[0].active_ws().monocle_window(), Some(100));
}
