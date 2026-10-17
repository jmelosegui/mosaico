use crate::event::Grab;
use crate::id::wid;
use crate::{Rect, WindowId};

use super::super::Drag;
use super::*;

/// A monitor 1000 wide at `x`, whose top 30 pixels are the bar's.
fn monitor(x: i32, tiles: Option<Vec<(WindowId, Rect)>>) -> DropArea {
    DropArea {
        area: Rect::new(x, 0, 1000, 800),
        tiles,
    }
}

/// Windows 1 and 2 side by side under the bar on monitor 0, window 3
/// alone on monitor 1, with gaps around them.
fn desk() -> Vec<DropArea> {
    let left = Rect::new(10, 40, 480, 750);
    let right = Rect::new(510, 40, 480, 750);
    let alone = Rect::new(1010, 40, 980, 750);
    vec![
        monitor(0, Some(vec![(wid(1), left), (wid(2), right)])),
        monitor(1000, Some(vec![(wid(3), alone)])),
    ]
}

/// Window 1, dragged from monitor 0 by `grab`, from its slot.
fn drag(grab: Option<Grab>) -> Drag {
    Drag {
        window: wid(1),
        monitor: 0,
        grab,
        from_maximized: false,
        start: Rect::new(10, 40, 480, 750),
        pointer: Some((250, 50)),
        left_start: true,
        decides: true,
    }
}

#[test]
fn a_window_dropped_on_another_tile_of_its_monitor_takes_its_slot() {
    // Act
    let lands = target(&drag(None), (700, 400), &desk());

    // Assert
    assert_eq!(lands, Some(Target::Swap(wid(2))));
}

#[test]
fn a_window_dropped_on_its_own_slot_or_a_gap_goes_back() {
    // Arrange -- its slot, the gap between the tiles, and the bar's strip.
    for pointer in [(200, 400), (500, 400), (700, 10)] {
        // Act
        let lands = target(&drag(None), pointer, &desk());

        // Assert
        assert_eq!(lands, Some(Target::Back), "at {pointer:?}");
    }
}

#[test]
fn a_window_dropped_on_another_monitor_goes_before_the_tile_under_the_pointer() {
    // Act
    let lands = target(&drag(None), (1500, 400), &desk());

    // Assert
    let before = Some(wid(3));
    assert_eq!(lands, Some(Target::Join { monitor: 1, before }));
}

#[test]
fn a_window_dropped_off_the_tiles_of_another_monitor_joins_the_end() {
    // Arrange -- a gap, and the bar's strip.
    for pointer in [(1995, 400), (1500, 10)] {
        // Act
        let lands = target(&drag(None), pointer, &desk());

        // Assert
        let before = None;
        assert_eq!(lands, Some(Target::Join { monitor: 1, before }));
    }
}

#[test]
fn a_monocle_workspace_has_no_tile_to_drop_on() {
    // Arrange -- both monitors in monocle.
    let desk: Vec<DropArea> = desk()
        .into_iter()
        .map(|area| DropArea {
            tiles: None,
            ..area
        })
        .collect();

    // Act
    let home = target(&drag(None), (700, 400), &desk);
    let away = target(&drag(None), (1500, 400), &desk);

    // Assert
    assert_eq!(home, Some(Target::Back));
    let before = None;
    assert_eq!(away, Some(Target::Join { monitor: 1, before }));
}

#[test]
fn a_pointer_off_every_monitor_lands_nowhere() {
    // Act -- below them, where a taskbar may be.
    let lands = target(&drag(None), (700, 850), &desk());

    // Assert
    assert_eq!(lands, None);
}
