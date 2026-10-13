use crate::action::MAX_WORKSPACES;
use crate::{EdidIdentity, LayoutKind, MonitorId, MonitorKey, Rect};

use crate::id::{wid, wids};

use super::super::Display;
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
/// stands in for the fresh session ids a platform hands out each time.
fn infos(keys: &[&MonitorKey], id_base: usize) -> Vec<Display> {
    keys.iter()
        .enumerate()
        .map(|(i, k)| Display {
            id: MonitorId::from_raw((id_base + i) as u64),
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
    mons[0].workspaces[0].add(wid(10));
    for window in [1, 2, 3] {
        mons[1].workspaces[0].add(wid(window));
    }
    mons[1].workspaces[2].add(wid(4));
    mons[1].workspaces[2].set_layout_kind(LayoutKind::VerticalStack);
    mons[1].workspaces[2].set_monocle(true);
    mons[1].workspaces[2].set_monocle_window(Some(wid(4)));
    mons[1].active_workspace = 2;

    // Act -- switch the KVM away, then back.
    let away = change(mons, &[&a], 50, &mut memory);
    let back = change(away, &[&a, &b], 90, &mut memory);

    // Assert
    assert_eq!(back[0].workspaces[0].windows(), wids(&[10]));
    assert!(back[0].workspaces[2].is_empty());
    assert_eq!(back[1].key, b);
    assert_eq!(back[1].workspaces[0].windows(), wids(&[1, 2, 3]));
    assert_eq!(back[1].workspaces[2].windows(), wids(&[4]));
    assert_eq!(back[1].active_workspace, 2);
    assert_eq!(
        back[1].workspaces[2].layout_kind(),
        LayoutKind::VerticalStack
    );
    assert_eq!(back[1].workspaces[2].monocle_window(), Some(wid(4)));
}

#[test]
fn windows_keep_their_workspace_number_while_away() {
    // Arrange
    let (a, b) = (key("A"), key("B"));
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&a, &b]);
    mons[1].workspaces[3].add(wid(7));

    // Act
    let away = change(mons, &[&a], 50, &mut memory);

    // Assert
    assert!(away[0].workspaces[3].contains(wid(7)));
}

#[test]
fn office_unplug_and_home_return_restores_both_externals() {
    // Arrange -- laptop plus two home monitors.
    let (laptop, left, right, office) = (key("L"), key("E1"), key("E2"), key("O"));
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&laptop, &left, &right]);
    mons[0].workspaces[0].add(wid(1));
    mons[1].workspaces[0].add(wid(2));
    mons[2].workspaces[1].add(wid(3));

    // Act -- undock, plug into an office monitor, come home.
    let undocked = change(mons, &[&laptop], 20, &mut memory);
    let at_office = change(undocked, &[&laptop, &office], 30, &mut memory);
    let home = change(at_office, &[&laptop, &left, &right], 40, &mut memory);

    // Assert
    assert_eq!(home[0].workspaces[0].windows(), wids(&[1]));
    assert_eq!(home[1].workspaces[0].windows(), wids(&[2]));
    assert_eq!(home[2].workspaces[1].windows(), wids(&[3]));
}

#[test]
fn window_moved_by_the_user_while_away_stays_put() {
    // Arrange
    let (a, b) = (key("A"), key("B"));
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&a, &b]);
    for window in [1, 2] {
        mons[1].workspaces[0].add(wid(window));
    }
    let mut away = change(mons, &[&a], 50, &mut memory);

    // Act -- the user sends window 2 to workspace 6, then reconnects.
    away[0].workspaces[0].remove(wid(2));
    away[0].workspaces[5].add(wid(2));
    let back = change(away, &[&a, &b], 90, &mut memory);

    // Assert
    assert_eq!(back[1].workspaces[0].windows(), wids(&[1]));
    assert!(back[0].workspaces[5].contains(wid(2)));
}

#[test]
fn window_closed_while_away_is_forgotten() {
    // Arrange
    let (a, b) = (key("A"), key("B"));
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&a, &b]);
    for window in [1, 2, 3] {
        mons[1].workspaces[0].add(wid(window));
    }
    mons[1].workspaces[0].set_monocle(true);
    mons[1].workspaces[0].set_monocle_window(Some(wid(2)));
    let mut away = change(mons, &[&a], 50, &mut memory);

    // Act -- window 2 is closed, then the monitor comes back.
    away[0].workspaces[0].remove(wid(2));
    memory.forget(wid(2));
    let back = change(away, &[&a, &b], 90, &mut memory);

    // Assert -- order kept, monocle on the closed window turned off.
    assert_eq!(back[1].workspaces[0].windows(), wids(&[1, 3]));
    assert!(!back[1].workspaces[0].monocle());
    assert!(memory.describe().ends_with("displaced windows: 0"));
}

#[test]
fn window_displaced_twice_still_returns_home() {
    // Arrange -- laptop and home monitor; then only an office monitor.
    let (laptop, home, office) = (key("L"), key("H"), key("O"));
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&laptop, &home]);
    mons[0].workspaces[0].add(wid(1));
    mons[1].workspaces[0].add(wid(2));

    // Act
    let undocked = change(mons, &[&laptop], 20, &mut memory);
    let office_only = change(undocked, &[&office], 30, &mut memory);
    let back = change(office_only, &[&laptop, &home], 40, &mut memory);

    // Assert
    assert_eq!(back[0].workspaces[0].windows(), wids(&[1]));
    assert_eq!(back[1].workspaces[0].windows(), wids(&[2]));
}

#[test]
fn moved_lists_every_window_that_changed_place() {
    // Arrange
    let (a, b) = (key("A"), key("B"));
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&a, &b]);
    mons[0].workspaces[0].add(wid(10));
    mons[1].workspaces[0].add(wid(1));
    mons[1].workspaces[1].add(wid(2));

    // Act
    let away = rebuild_monitors(mons, &infos(&[&a], 50), &layouts(), &mut memory);

    // Assert
    let mut moved = away.moved;
    moved.sort();
    assert_eq!(moved, wids(&[1, 2]));
}

/// Full key as Windows reports it for a monitor that is really there.
fn full(model: u16, serial: u32, connector: &str) -> MonitorKey {
    MonitorKey {
        stable_id: Some(format!("path-{model:X}-{connector}")),
        edid: Some(EdidIdentity {
            manufacturer: "DEL".into(),
            product: model,
            serial: Some(serial),
        }),
        connector: Some(connector.into()),
    }
}

/// What the same monitor looks like while the KVM is switched away: an
/// emulated display with the model ids but no device path or serial.
fn degraded(model: u16, connector: &str) -> MonitorKey {
    MonitorKey {
        stable_id: None,
        edid: Some(EdidIdentity {
            manufacturer: "DEL".into(),
            product: model,
            serial: None,
        }),
        connector: Some(connector.into()),
    }
}

#[test]
fn kvm_with_degraded_key_while_away_still_restores() {
    // Arrange -- the log of a real KVM switch: the left monitor stays,
    // but reports a reduced key while the KVM points elsewhere.
    let left = full(0xA0E9, 808_726_860, "hdmi:0");
    let right = full(0xA0EC, 809_847_884, "dp:1");
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&left, &right]);
    mons[0].workspaces[0].add(wid(1));
    mons[1].workspaces[0].add(wid(2));

    // Act
    let away = change(mons, &[&degraded(0xA0E9, "hdmi:0")], 50, &mut memory);
    let back = change(away, &[&left, &right], 90, &mut memory);

    // Assert
    assert_eq!(back[0].workspaces[0].windows(), wids(&[1]));
    assert_eq!(back[1].workspaces[0].windows(), wids(&[2]));
}

#[test]
fn monitor_returning_with_a_degraded_key_finds_its_parked_state() {
    // Arrange
    let left = full(0xA0E9, 1, "hdmi:0");
    let right = full(0xA0EC, 2, "dp:1");
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&left, &right]);
    mons[1].workspaces[2].add(wid(7));
    mons[1].active_workspace = 2;

    // Act -- it comes back before Windows has read its full identity.
    let away = change(mons, &[&left], 50, &mut memory);
    let back = change(away, &[&left, &degraded(0xA0EC, "dp:1")], 90, &mut memory);

    // Assert
    assert_eq!(back[1].workspaces[2].windows(), wids(&[7]));
    assert_eq!(back[1].active_workspace, 2);
}

#[test]
fn window_on_an_unidentified_host_still_returns() {
    // Arrange -- while away the only display cannot be identified.
    let (left, right) = (full(0xA0E9, 1, "hdmi:0"), full(0xA0EC, 2, "dp:1"));
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&left, &right]);
    mons[1].workspaces[0].add(wid(2));

    // Act
    let away = change(mons, &[&MonitorKey::default()], 50, &mut memory);
    let back = change(away, &[&left, &right], 90, &mut memory);

    // Assert
    assert_eq!(back[1].workspaces[0].windows(), wids(&[2]));
}

#[test]
fn unidentified_monitor_is_not_remembered() {
    // Arrange
    let a = key("A");
    let unknown = MonitorKey::default();
    let mut memory = ReconnectMemory::default();
    let mut mons = states(&[&a, &unknown]);
    mons[1].workspaces[0].add(wid(1));

    // Act
    let away = change(mons, &[&a], 50, &mut memory);

    // Assert -- the window still moves, but nothing waits to bring it back.
    assert!(away[0].workspaces[0].contains(wid(1)));
    assert_eq!(
        memory.describe(),
        "Disconnected monitors: none, displaced windows: 0"
    );
}
