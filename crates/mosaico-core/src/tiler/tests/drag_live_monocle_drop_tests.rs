//! Drops onto a monocle workspace, as the owner chose while testing: one
//! over another monitor's bar leaves monocle there and joins its end, its
//! monocle window goes back to fill the monitor, and the others to where
//! monocle left them, however they were as the drag began, when the drop
//! goes elsewhere, while a window there minimized without a report stays
//! so, and, as before, a drop with `drag_to_swap` off joins its end in
//! monocle, and one on the window's own monocle workspace goes back.

use crate::id::wid;
use crate::tiler::Settings;
use crate::tiler::drag::live::support::{FRONT, RIGHT, monocle_on_second};
use crate::tiler::drag::support::{center, drag_to, grab, report_at, reserve_bar, swapping};
use crate::tiler::tests::{Harness, MON_W};
use crate::{Action, Rect};

#[test]
fn a_window_dropped_over_the_bar_of_a_monocle_workspace_joins_its_end_tiled() {
    // Arrange -- a 30 pixel bar on every monitor.
    let mut h = monocle_on_second(swapping(), &[3, 4]);
    reserve_bar(&mut h);

    // Act
    drag_to(&mut h, 1, (MON_W + 100, 15));

    // Assert -- monocle ends there, and window 1 is tiled after window 4.
    let ws = h.workspace(1, 0);
    assert!(!ws.monocle() && ws.monocle_window().is_none());
    assert_eq!(h.active(1), vec![3, 4, 1]);
    assert_eq!(h.rect_of(1), Rect::new(MON_W + 964, 559, 948, 513));
    assert_eq!(h.rect_of(3), Rect::new(MON_W + 8, 38, 948, 1034));
}

#[test]
fn monocle_comes_back_with_its_window_filling_the_monitor_and_the_others_where_it_left_them() {
    // Arrange -- window 3 still on its way to fill the monitor, and window
    // 4 behind it off its tiled slot, where monocle left it.
    let mut h = monocle_on_second(swapping(), &[3, 4]);
    let home = center(h.rect_of(1));
    let left_there = Rect::new(MON_W + 300, 200, 700, 500);
    h.window(3).rect = Rect::new(MON_W + 100, 100, 800, 600);
    h.window(4).rect = left_there;
    grab(&mut h, 1);
    report_at(&mut h, 1, center(RIGHT));

    // Act
    report_at(&mut h, 1, home);

    // Assert
    assert_eq!((h.rect_of(3), h.rect_of(4)), (FRONT, left_there));
}

#[test]
fn a_window_there_minimized_without_a_report_stays_so_as_monocle_comes_back() {
    // Arrange -- windows 3 and 4 laid out tiled, then window 4 minimized
    // without a report, which the next report finds.
    let mut h = monocle_on_second(swapping(), &[3, 4]);
    let (home, (x, y)) = (center(h.rect_of(1)), center(RIGHT));
    grab(&mut h, 1);
    report_at(&mut h, 1, (x, y));
    h.window(4).minimized = true;
    report_at(&mut h, 1, (x + 5, y));

    // Act
    report_at(&mut h, 1, home);

    // Assert -- window 3 fills the monitor again, and nothing places
    // window 4.
    assert_eq!(h.rect_of(3), FRONT);
    assert!(h.placements().iter().all(|p| p.0 != 4), "{:?}", h.effects);
    assert!(h.window(4).minimized);
}

#[test]
fn with_drag_to_swap_off_a_window_dropped_on_a_monocle_workspace_joins_its_end_as_before() {
    // Arrange
    let mut h = monocle_on_second(Settings::with_gap(8), &[3, 4]);

    // Act
    drag_to(&mut h, 1, center(RIGHT));

    // Assert -- monocle stays, on window 3.
    let ws = h.workspace(1, 0);
    assert!(ws.monocle());
    assert_eq!(ws.monocle_window(), Some(wid(3)));
    assert_eq!(h.active(1), vec![3, 4, 1]);
    assert_eq!(h.rect_of(3), FRONT);
}

#[test]
fn a_window_dropped_on_its_own_monocle_workspace_goes_back_as_before() {
    // Arrange -- window 1 fills the monitor in monocle, window 2 behind.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let behind = h.rect_of(2);
    h.action(Action::ToggleMonocle);
    let front = h.rect_of(1);

    // Act -- over where window 2 is tiled.
    drag_to(&mut h, 1, center(behind));

    // Assert -- no swap, and monocle stays.
    assert_eq!(h.active(0), vec![1, 2]);
    assert!(h.workspace(0, 0).monocle());
    assert_eq!(h.rect_of(1), front);
}
