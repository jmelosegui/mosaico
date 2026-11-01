//! Drops with `drag_to_swap` off, which put the window back as before, and
//! the windows the tiler does not follow the drag of.

use crate::WindowEvent;
use crate::id::wid;
use crate::tiler::Settings;
use crate::tiler::tests::Harness;
use crate::tiler::tests::fake_desktop::FakeWindow;

use super::super::support::{drag_to, drop_at, places, swapping};

#[test]
fn with_drag_to_swap_off_a_window_dropped_on_another_goes_back() {
    // Arrange
    let mut h = Harness::with_settings(Settings::with_gap(8), 1, &[(1, 0), (2, 0)]);
    let (left, right) = (h.rect_of(1), h.rect_of(2));

    // Act
    drag_to(&mut h, 1, (right.center_x(), right.center_y()));

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!(h.rect_of(1), left);
}

#[test]
fn with_drag_to_swap_off_a_window_dropped_on_another_monitor_joins_the_end() {
    // Arrange
    let windows = [(1, 0), (2, 0), (3, 1), (4, 1)];
    let mut h = Harness::with_settings(Settings::with_gap(8), 2, &windows);
    let left = h.rect_of(3);

    // Act -- onto window 3, the first there.
    drag_to(&mut h, 1, (left.center_x(), left.center_y()));

    // Assert
    assert_eq!(h.active(1), vec![3, 4, 1]);
}

#[test]
fn the_tiles_stay_put_under_a_window_the_tiler_leaves_alone() {
    // Arrange -- a window no rule manages, dragged over window 2.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let mut floating = FakeWindow::app(0);
    floating.facts = None;
    h.desktop.windows.insert(wid(9), floating);
    let right = h.rect_of(2);

    // Act
    h.event(WindowEvent::move_started(wid(9)));
    h.window(9).rect = right;
    let effects = drop_at(&mut h, 9, (right.center_x(), right.center_y()));

    // Assert
    assert!(effects.is_empty(), "{effects:?}");
    assert_eq!(h.active(0), vec![1, 2]);
}

#[test]
fn a_window_the_tiler_does_not_manage_yet_is_taken_in_where_it_is_dropped() {
    // Arrange -- a window it would tile, never reported, dragged onto the
    // second monitor, over window 3.
    let mut h = Harness::with_settings(swapping(), 2, &[(1, 0), (3, 1)]);
    h.desktop.windows.insert(wid(9), FakeWindow::app(0));
    let over = h.rect_of(3);

    // Act
    h.event(WindowEvent::move_started(wid(9)));
    h.window(9).monitor = 1;
    let effects = drop_at(&mut h, 9, (over.center_x(), over.center_y()));

    // Assert -- as before, at the end.
    assert_eq!(h.active(1), vec![3, 9]);
    assert!(places(&effects, 9), "{effects:?}");
}
