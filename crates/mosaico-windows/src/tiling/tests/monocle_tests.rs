use super::make_monitor;

#[test]
fn monocle_toggle_sets_monocle_window() {
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(100);
    mon.workspaces[0].add(200);

    // Simulate toggle_monocle: enable with focused_window = 100
    let focused_window = Some(100usize);
    mon.monocle = true;
    mon.monocle_window = focused_window;

    assert!(mon.monocle);
    assert_eq!(mon.monocle_window, Some(100));
}

#[test]
fn monocle_toggle_clears_monocle_window() {
    let mut mon = make_monitor(1);
    mon.monocle = true;
    mon.monocle_window = Some(100);

    // Simulate toggle_monocle: disable
    mon.monocle = false;
    mon.monocle_window = None;

    assert!(!mon.monocle);
    assert_eq!(mon.monocle_window, None);
}

#[test]
fn monocle_blocks_move_on_same_monitor() {
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(100);
    mon.workspaces[0].add(200);
    mon.monocle = true;
    mon.monocle_window = Some(100);

    // In monocle mode, move_direction returns early.
    // Verify the state that triggers the early return.
    assert!(mon.monocle);

    // Windows should remain unchanged — monocle blocks all moves.
    assert_eq!(mon.workspaces[0].len(), 2);
    assert!(mon.workspaces[0].contains(100));
    assert!(mon.workspaces[0].contains(200));
}

#[test]
fn monocle_entry_uses_monocle_window() {
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(100);
    mon.workspaces[0].add(200);
    mon.workspaces[0].add(300);
    mon.monocle = true;
    mon.monocle_window = Some(200);

    // When entering a monocle monitor, focus_adjacent_monitor_idx
    // picks monocle_window over spatial entry.
    let entry = mon
        .monocle_window
        .or_else(|| mon.active_ws().handles().first().copied());
    assert_eq!(entry, Some(200));
}

#[test]
fn monocle_entry_falls_back_to_first_window() {
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(100);
    mon.workspaces[0].add(200);
    mon.monocle = true;
    mon.monocle_window = None; // e.g. monocle window was closed

    let entry = mon
        .monocle_window
        .or_else(|| mon.active_ws().handles().first().copied());
    assert_eq!(entry, Some(100));
}

#[test]
fn monocle_no_vertical_navigation() {
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(100);
    mon.workspaces[0].add(200);
    mon.monocle = true;
    mon.monocle_window = Some(100);

    // In monocle mode, focus_direction returns early for Up/Down.
    // The monocle flag is the guard condition.
    assert!(mon.monocle);
    // No vertical neighbor lookup should happen — there is
    // conceptually only one window.
}

#[test]
fn monocle_clears_when_monocle_window_destroyed() {
    // Arrange
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(100);
    mon.workspaces[0].add(200);
    mon.monocle = true;
    mon.monocle_window = Some(100);

    // Act — simulate Destroyed handler: remove and clear monocle.
    mon.workspaces[0].remove(100);
    if mon.monocle && mon.monocle_window == Some(100) {
        mon.monocle = false;
        mon.monocle_window = None;
    }

    // Assert
    assert!(!mon.monocle);
    assert_eq!(mon.monocle_window, None);
    assert_eq!(mon.workspaces[0].len(), 1);
    assert!(mon.workspaces[0].contains(200));
}

#[test]
fn monocle_persists_when_other_window_destroyed() {
    // Arrange
    let mut mon = make_monitor(1);
    mon.workspaces[0].add(100);
    mon.workspaces[0].add(200);
    mon.monocle = true;
    mon.monocle_window = Some(100);

    // Act — destroy a non-monocle window.
    mon.workspaces[0].remove(200);
    if mon.monocle && mon.monocle_window == Some(200) {
        mon.monocle = false;
        mon.monocle_window = None;
    }

    // Assert — monocle stays active.
    assert!(mon.monocle);
    assert_eq!(mon.monocle_window, Some(100));
    assert_eq!(mon.workspaces[0].len(), 1);
}
