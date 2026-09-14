use super::super::layout::monocle_rect_for;
use super::make_monitor;
use crate::tiling::helpers::should_pre_place;
use mosaico_core::Rect;

// -- should_pre_place --

#[test]
fn pre_places_a_brand_new_window() {
    assert!(should_pre_place(false, true));
}

#[test]
fn skips_pre_placement_for_a_window_already_tracked() {
    // Child element events share the parent hwnd, so the same window
    // can reach the Created handler more than once.
    assert!(!should_pre_place(true, true));
}

#[test]
fn skips_pre_placement_when_rules_reject_the_window() {
    assert!(!should_pre_place(false, false));
}

// -- the add and remove round trip pre_place relies on --
//
// pre_place appends the hwnd, asks the layout what it would produce,
// then removes it again. If that round trip did not restore the
// workspace exactly, every hidden window an application creates would
// disturb the real windows around it.

#[test]
fn add_then_remove_restores_the_handle_list() {
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(100);
    mon.workspaces[0].add(200);
    let before: Vec<usize> = mon.workspaces[0].handles().to_vec();

    assert!(mon.active_ws_mut().add(300));
    assert_eq!(mon.active_ws().len(), 3);
    mon.active_ws_mut().remove(300);

    assert_eq!(mon.workspaces[0].handles().to_vec(), before);
    assert!(!mon.workspaces[0].contains(300));
}

#[test]
fn add_then_remove_leaves_last_focused_alone() {
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(100);
    mon.active_ws_mut().set_last_focused(Some(100));

    mon.active_ws_mut().add(300);
    mon.active_ws_mut().remove(300);

    // remove() only clears last_focused when it matches the handle
    // being removed, and a window that was never shown was never
    // focused, so the promotion target on close is unaffected.
    assert_eq!(mon.active_ws().last_focused(), Some(100));
}

#[test]
fn a_window_that_is_already_tracked_is_not_added_twice() {
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(100);

    // pre_place bails out when add() reports the window is already
    // managed, so it never removes a handle it did not add.
    assert!(!mon.active_ws_mut().add(100));
    assert_eq!(mon.active_ws().len(), 1);
}

// -- monocle pre placement --
//
// A window adopted onto a monocle workspace becomes the monocle target
// and fills the work area, so its destination is knowable before it is
// shown. That is what lets pre placement cover monocle at all: asking
// the layout instead would return a tiled slot that nothing ever moves
// the window into.

#[test]
fn monocle_fills_the_work_area_inset_by_the_gap() {
    // Arrange
    let work_area = Rect::new(0, 64, 3840, 2024);

    // Act
    let rect = monocle_rect_for(&work_area, 20);

    // Assert
    assert_eq!(rect, Rect::new(20, 84, 3800, 1984));
}

#[test]
fn monocle_with_no_gap_covers_the_whole_work_area() {
    // Arrange
    let work_area = Rect::new(3840, 64, 3840, 2024);

    // Act
    let rect = monocle_rect_for(&work_area, 0);

    // Assert
    assert_eq!(rect, work_area);
}

#[test]
fn monocle_never_produces_a_collapsed_rect() {
    // Arrange -- a gap wider than the monitor, which the config
    // validator allows up to its own clamp.
    let work_area = Rect::new(0, 0, 100, 80);

    // Act
    let rect = monocle_rect_for(&work_area, 400);

    // Assert -- a zero or negative size would be rejected by
    // SetWindowPos, so the arithmetic floors at one pixel.
    assert!(rect.width >= 1, "width collapsed to {}", rect.width);
    assert!(rect.height >= 1, "height collapsed to {}", rect.height);
}

#[test]
fn monocle_pre_placement_is_no_longer_excluded() {
    // Monocle used to be rejected before reaching pre_place. It is now
    // handled there by asking monocle_rect instead of the layout, so
    // the caller no longer decides based on the mode.
    assert!(should_pre_place(false, true));
}
