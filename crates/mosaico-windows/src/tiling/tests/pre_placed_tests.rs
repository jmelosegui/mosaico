use super::make_monitor;
use crate::tiling::helpers::should_pre_place;

// -- should_pre_place --

#[test]
fn pre_places_a_brand_new_window() {
    assert!(should_pre_place(false, false, true));
}

#[test]
fn skips_pre_placement_in_monocle() {
    // compute_positions lays out every window in the workspace, while
    // monocle only ever shows one, so the rect it produces is not
    // where the window would end up.
    assert!(!should_pre_place(true, false, true));
}

#[test]
fn skips_pre_placement_for_a_window_already_tracked() {
    // Child element events share the parent hwnd, so the same window
    // can reach the Created handler more than once.
    assert!(!should_pre_place(false, true, true));
}

#[test]
fn skips_pre_placement_when_rules_reject_the_window() {
    assert!(!should_pre_place(false, false, false));
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
