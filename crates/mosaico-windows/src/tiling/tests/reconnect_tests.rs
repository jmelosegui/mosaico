use mosaico_core::action::MAX_WORKSPACES;
use mosaico_core::{LayoutKind, MonitorKey, Rect};

use crate::monitor::MonitorInfo;

use super::super::MonitorState;
use super::super::display_rebuild::rebuild_monitors;
use super::ReconnectMemory;

fn key(name: &str) -> MonitorKey {
    MonitorKey {
        stable_id: Some(name.into()),
        ..Default::default()
    }
}

fn layouts() -> Vec<LayoutKind> {
    vec![LayoutKind::default(); MAX_WORKSPACES as usize]
}

/// Monitors laid out left to right, the first one primary. `id_base`
/// stands in for the fresh HMONITOR values Windows hands out each time.
fn infos(keys: &[&MonitorKey], id_base: usize) -> Vec<MonitorInfo> {
    keys.iter()
        .enumerate()
        .map(|(i, k)| MonitorInfo {
            id: id_base + i,
            key: (*k).clone(),
            work_area: Rect::new(i as i32 * 1920, 0, 1920, 1080),
            primary: i == 0,
        })
        .collect()
}

fn states(keys: &[&MonitorKey]) -> Vec<MonitorState> {
    infos(keys, 1)
        .iter()
        .map(|i| MonitorState::new(i, &layouts()))
        .collect()
}

fn change(
    old: Vec<MonitorState>,
    keys: &[&MonitorKey],
    id_base: usize,
    memory: &mut ReconnectMemory,
) -> Vec<MonitorState> {
    rebuild_monitors(old, &infos(keys, id_base), &layouts(), memory).monitors
}

#[test]
fn kvm_round_trip_restores_monitor_workspace_and_order() {
    // Arrange
    let (a, b) = (key("A"), key("B"));
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&a, &b]);
    mons[0].workspaces[0].add(10);
    for hwnd in [1, 2, 3] {
        mons[1].workspaces[0].add(hwnd);
    }
    mons[1].workspaces[2].add(4);
    mons[1].workspaces[2].set_layout_kind(LayoutKind::VerticalStack);
    mons[1].workspaces[2].set_monocle(true);
    mons[1].workspaces[2].set_monocle_window(Some(4));
    mons[1].active_workspace = 2;

    // Act -- switch the KVM away, then back.
    let away = change(mons, &[&a], 50, &mut memory);
    let back = change(away, &[&a, &b], 90, &mut memory);

    // Assert
    assert_eq!(back[0].workspaces[0].handles(), &[10]);
    assert!(back[0].workspaces[2].is_empty());
    assert_eq!(back[1].key, b);
    assert_eq!(back[1].workspaces[0].handles(), &[1, 2, 3]);
    assert_eq!(back[1].workspaces[2].handles(), &[4]);
    assert_eq!(back[1].active_workspace, 2);
    assert_eq!(
        back[1].workspaces[2].layout_kind(),
        LayoutKind::VerticalStack
    );
    assert_eq!(back[1].workspaces[2].monocle_window(), Some(4));
}

#[test]
fn windows_keep_their_workspace_number_while_away() {
    // Arrange
    let (a, b) = (key("A"), key("B"));
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&a, &b]);
    mons[1].workspaces[3].add(7);

    // Act
    let away = change(mons, &[&a], 50, &mut memory);

    // Assert
    assert!(away[0].workspaces[3].contains(7));
}

#[test]
fn office_unplug_and_home_return_restores_both_externals() {
    // Arrange -- laptop plus two home monitors.
    let (laptop, left, right, office) = (key("L"), key("E1"), key("E2"), key("O"));
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&laptop, &left, &right]);
    mons[0].workspaces[0].add(1);
    mons[1].workspaces[0].add(2);
    mons[2].workspaces[1].add(3);

    // Act -- undock, plug into an office monitor, come home.
    let undocked = change(mons, &[&laptop], 20, &mut memory);
    let at_office = change(undocked, &[&laptop, &office], 30, &mut memory);
    let home = change(at_office, &[&laptop, &left, &right], 40, &mut memory);

    // Assert
    assert_eq!(home[0].workspaces[0].handles(), &[1]);
    assert_eq!(home[1].workspaces[0].handles(), &[2]);
    assert_eq!(home[2].workspaces[1].handles(), &[3]);
}

#[test]
fn window_moved_by_the_user_while_away_stays_put() {
    // Arrange
    let (a, b) = (key("A"), key("B"));
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&a, &b]);
    for hwnd in [1, 2] {
        mons[1].workspaces[0].add(hwnd);
    }
    let mut away = change(mons, &[&a], 50, &mut memory);

    // Act -- the user sends window 2 to workspace 6, then reconnects.
    away[0].workspaces[0].remove(2);
    away[0].workspaces[5].add(2);
    let back = change(away, &[&a, &b], 90, &mut memory);

    // Assert
    assert_eq!(back[1].workspaces[0].handles(), &[1]);
    assert!(back[0].workspaces[5].contains(2));
}

#[test]
fn window_closed_while_away_is_forgotten() {
    // Arrange
    let (a, b) = (key("A"), key("B"));
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&a, &b]);
    for hwnd in [1, 2, 3] {
        mons[1].workspaces[0].add(hwnd);
    }
    mons[1].workspaces[0].set_monocle(true);
    mons[1].workspaces[0].set_monocle_window(Some(2));
    let mut away = change(mons, &[&a], 50, &mut memory);

    // Act -- window 2 is closed, then the monitor comes back.
    away[0].workspaces[0].remove(2);
    memory.forget(2);
    let back = change(away, &[&a, &b], 90, &mut memory);

    // Assert -- order kept, monocle on the closed window turned off.
    assert_eq!(back[1].workspaces[0].handles(), &[1, 3]);
    assert!(!back[1].workspaces[0].monocle());
    assert!(memory.describe().ends_with("displaced windows: 0"));
}

#[test]
fn window_displaced_twice_still_returns_home() {
    // Arrange -- laptop and home monitor; then only an office monitor.
    let (laptop, home, office) = (key("L"), key("H"), key("O"));
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&laptop, &home]);
    mons[0].workspaces[0].add(1);
    mons[1].workspaces[0].add(2);

    // Act
    let undocked = change(mons, &[&laptop], 20, &mut memory);
    let office_only = change(undocked, &[&office], 30, &mut memory);
    let back = change(office_only, &[&laptop, &home], 40, &mut memory);

    // Assert
    assert_eq!(back[0].workspaces[0].handles(), &[1]);
    assert_eq!(back[1].workspaces[0].handles(), &[2]);
}

#[test]
fn moved_lists_every_window_that_changed_place() {
    // Arrange
    let (a, b) = (key("A"), key("B"));
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&a, &b]);
    mons[0].workspaces[0].add(10);
    mons[1].workspaces[0].add(1);
    mons[1].workspaces[1].add(2);

    // Act
    let away = rebuild_monitors(mons, &infos(&[&a], 50), &layouts(), &mut memory);

    // Assert
    let mut moved = away.moved;
    moved.sort();
    assert_eq!(moved, vec![1, 2]);
}

#[test]
fn unidentified_monitor_is_not_remembered() {
    // Arrange
    let a = key("A");
    let unknown = MonitorKey::default();
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&a, &unknown]);
    mons[1].workspaces[0].add(1);

    // Act
    let away = change(mons, &[&a], 50, &mut memory);

    // Assert -- the window still moves, but nothing waits to bring it back.
    assert!(away[0].workspaces[0].contains(1));
    assert_eq!(
        memory.describe(),
        "Disconnected monitors: none, displaced windows: 0"
    );
}
