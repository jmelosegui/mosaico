use super::super::work_area::{apply_reach, below_bar, sync_os_work_areas};
use super::super::*;
use super::{MON_H, MON_W, make_monitors, simulate_display_change};
use crate::MonitorId;
use crate::id::wid;
use crate::{MonitorKey, Rect};

const BAR: i32 = 44;

/// Keeps a bars height clear on the monitors at `indices`, or on every
/// one when it is empty.
fn reserve(monitors: &mut [MonitorState], indices: &[usize]) {
    let mut reach = Reach::none();
    for (i, mon) in monitors.iter().enumerate() {
        if indices.is_empty() || indices.contains(&i) {
            reach.set_top(mon.stable_id.clone(), BAR);
        }
    }
    apply_reach(monitors, &reach);
}

fn os_monitor(id: usize, x: i32) -> Display {
    Display {
        id: MonitorId::from_raw(id as u64),
        key: MonitorKey::default(),
        bounds: Rect::new(x, 0, 2560, 1392),
        work_area: Rect::new(x, 0, 2560, 1392),
        primary: x == 0,
        name: String::new(),
        device: String::new(),
        scale: 1.0,
    }
}

#[test]
fn below_bar_removes_bar_from_top() {
    // Arrange
    let area = Rect::new(2560, 0, 2560, 1392);

    // Act
    let result = below_bar(&area, BAR);

    // Assert
    assert_eq!(result, Rect::new(2560, 44, 2560, 1348));
}

#[test]
fn repeated_bar_reservation_does_not_accumulate() {
    // Arrange: taskbar show/hide fires WorkAreaChanged several times.
    let mut monitors = make_monitors(1);

    // Act
    for _ in 0..3 {
        reserve(&mut monitors, &[0]);
    }

    // Assert: still exactly one bar height, not three.
    assert_eq!(monitors[0].work_area, Rect::new(0, 44, 1920, 1036));
}

#[test]
fn bar_reservation_skips_monitors_without_bar() {
    // Arrange
    let mut monitors = make_monitors(2);

    // Act
    reserve(&mut monitors, &[1]);

    // Assert
    assert_eq!(monitors[0].work_area, monitors[0].os_work_area);
    assert_eq!(monitors[1].work_area.y, 44);
}

#[test]
fn bar_reservation_with_empty_list_covers_all_monitors() {
    // Arrange
    let mut monitors = make_monitors(2);

    // Act
    reserve(&mut monitors, &[]);

    // Assert
    assert!(monitors.iter().all(|m| m.work_area.y == 44));
}

#[test]
fn sync_matches_by_id() {
    // Arrange
    let mut monitors = make_monitors(2);
    let os = vec![os_monitor(1, 2560), os_monitor(0, 0)];

    // Act
    sync_os_work_areas(&mut monitors, &os);

    // Assert
    assert_eq!(monitors[0].os_work_area.x, 0);
    assert_eq!(monitors[1].os_work_area.x, 2560);
}

#[test]
fn sync_falls_back_to_index_when_ids_change() {
    // Arrange: a KVM reconnect gives both monitors new session ids.
    let mut monitors = make_monitors(2);
    reserve(&mut monitors, &[]);
    let os = vec![os_monitor(100, 0), os_monitor(200, 2560)];

    // Act
    sync_os_work_areas(&mut monitors, &os);
    reserve(&mut monitors, &[]);

    // Assert: fresh OS geometry and IDs, bar reserved exactly once.
    assert_eq!(monitors[0].id, MonitorId::from_raw(100));
    assert_eq!(monitors[1].id, MonitorId::from_raw(200));
    assert_eq!(monitors[1].work_area, Rect::new(2560, 44, 2560, 1348));
}

#[test]
fn sync_keeps_unmatched_monitors_when_count_differs() {
    // Arrange: a display change is pending, the counts disagree.
    let mut monitors = make_monitors(2);
    let os = vec![os_monitor(100, 0)];

    // Act
    sync_os_work_areas(&mut monitors, &os);

    // Assert: nothing is guessed; the display change handler owns this.
    assert_eq!(monitors[0].id, MonitorId::from_raw(0));
    assert_eq!(monitors[1].id, MonitorId::from_raw(1));
    assert_eq!(monitors[0].os_work_area, Rect::new(0, 0, 1920, 1080));
}

#[test]
fn display_change_matches_bar_monitor_by_position_when_id_changes() {
    // Arrange: the tiling area sits below the bar, the OS area does not.
    let mut monitors = make_monitors(1);
    monitors[0].workspaces[0].add(wid(100));
    reserve(&mut monitors, &[0]);
    let new_infos = vec![(42, Rect::new(0, 0, 1920, 1080))];

    // Act
    let result = simulate_display_change(&mut monitors, new_infos);

    // Assert: matched by position, so the window stays put.
    assert_eq!(result[0].id, MonitorId::from_raw(42));
    assert!(result[0].active_ws().contains(wid(100)));
}

#[test]
fn a_reach_the_system_already_keeps_clear_is_not_counted_twice() {
    // Arrange -- a bar 30 tall that the system already leaves out of the
    // work area, as a bar registered with it is.
    let mut monitors = make_monitors(1);
    monitors[0].os_work_area = Rect::new(0, 30, MON_W, MON_H - 30);
    let mut reach = Reach::none();
    reach.set_top(monitors[0].stable_id.clone(), 30);

    // Act
    apply_reach(&mut monitors, &reach);

    // Assert
    assert_eq!(monitors[0].work_area, Rect::new(0, 30, MON_W, MON_H - 30));
}

#[test]
fn a_reach_past_what_the_system_keeps_clear_takes_the_rest() {
    // Arrange -- a taskbar of 40 at the top, and a bar drawn below it down
    // to 70 from the monitor's edge.
    let mut monitors = make_monitors(1);
    monitors[0].os_work_area = Rect::new(0, 40, MON_W, MON_H - 40);
    let mut reach = Reach::none();
    reach.set_top(monitors[0].stable_id.clone(), 70);

    // Act
    apply_reach(&mut monitors, &reach);

    // Assert
    assert_eq!(monitors[0].work_area, Rect::new(0, 70, MON_W, MON_H - 70));
}
