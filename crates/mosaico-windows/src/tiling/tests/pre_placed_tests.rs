use std::time::Duration;

use super::make_monitor;
use crate::tiling::helpers::{PrePlacedAction, pre_placed_action, should_pre_place};

const TTL: Duration = Duration::from_secs(2);

// -- should_pre_place --

#[test]
fn pre_places_a_brand_new_window() {
    assert!(should_pre_place(false, false, true));
}

#[test]
fn skips_pre_placement_in_monocle() {
    // Monocle positions only the monocle window, so a pre placed
    // window would hold a slot the layout never moves.
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

// -- pre_placed_action --

#[test]
fn keeps_an_entry_inside_the_ttl() {
    assert_eq!(
        pre_placed_action(Duration::from_millis(500), TTL, false),
        PrePlacedAction::Keep
    );
    // Still keeps it even when the window is already usable: the show
    // event is what completes the adoption.
    assert_eq!(
        pre_placed_action(Duration::from_millis(500), TTL, true),
        PrePlacedAction::Keep
    );
}

#[test]
fn settles_an_expired_entry_whose_window_is_on_screen() {
    // The window showed up and still passes the rules, so its slot is
    // real and only the tracking entry goes away.
    assert_eq!(
        pre_placed_action(Duration::from_secs(3), TTL, true),
        PrePlacedAction::Settle
    );
}

#[test]
fn reclaims_an_expired_entry_whose_window_never_appeared() {
    assert_eq!(
        pre_placed_action(Duration::from_secs(3), TTL, false),
        PrePlacedAction::Reclaim
    );
}

#[test]
fn treats_the_ttl_boundary_as_expired() {
    assert_eq!(pre_placed_action(TTL, TTL, false), PrePlacedAction::Reclaim);
}

// -- workspace bookkeeping the sweep relies on --

#[test]
fn reclaiming_a_slot_leaves_the_other_windows_alone() {
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(100);
    mon.workspaces[0].add(200);
    mon.workspaces[0].add(300);

    // Simulate the Reclaim arm of sweep_pre_placed for hwnd 200.
    mon.workspaces[0].remove(200);

    assert!(mon.workspaces[0].contains(100));
    assert!(!mon.workspaces[0].contains(200));
    assert!(mon.workspaces[0].contains(300));
    assert_eq!(mon.workspaces[0].len(), 2);
}

#[test]
fn a_pre_placed_window_occupies_a_real_slot_until_it_is_reclaimed() {
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(100);

    // pre_place adds the window to the active workspace even though
    // Windows has not shown it yet, so the layout accounts for it.
    mon.active_ws_mut().add(200);
    assert_eq!(mon.active_ws().len(), 2);

    // The sweep gives the slot back when the window never appears.
    mon.active_ws_mut().remove(200);
    assert_eq!(mon.active_ws().len(), 1);
    assert!(mon.active_ws().contains(100));
}
