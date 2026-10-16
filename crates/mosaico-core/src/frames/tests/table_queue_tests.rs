//! When the table has a window's other changes posted behind what may be
//! queued for it, instead of made at once.

use super::*;

/// Whether a change made to window 1 now is posted behind.
fn posts_behind(table: &mut FrameTable<i32>, probe: &Probe) -> bool {
    table.posts_behind(wid(1), probe, probe.now)
}

#[test]
fn a_window_nothing_was_posted_to_is_changed_at_once() {
    // Arrange
    let mut table = FrameTable::default();
    let probe = Probe::new(true);

    // Act / Assert
    assert!(!posts_behind(&mut table, &probe));
}

#[test]
fn a_change_made_after_a_frame_that_landed_is_posted_behind_it_for_the_grace() {
    // Arrange -- the window is seen where the frame put it.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    place(&mut table, &mut probe, frame(120, true));

    // Act
    let at_once = posts_behind(&mut table, &probe);
    probe.now += ASYNC_GRACE - PENDING_LIMIT;
    let within = posts_behind(&mut table, &probe);
    probe.now += PENDING_LIMIT;
    let after = posts_behind(&mut table, &probe);

    // Assert
    assert!(at_once && within);
    assert!(!after);
}

#[test]
fn a_change_made_while_the_last_frame_may_be_queued_is_posted_past_the_grace() {
    // Arrange -- the window has not moved: the frame is still queued, or
    // was taken without moving.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(false);
    place(&mut table, &mut probe, frame(120, true));

    // Act
    probe.now += ASYNC_GRACE * 4;
    let unmoved = posts_behind(&mut table, &probe);
    probe.window().at = at(120);
    let moved = posts_behind(&mut table, &probe);

    // Assert -- until the window is seen to move.
    assert!(unmoved);
    assert!(!moved);
}

#[test]
fn a_window_seen_to_move_is_changed_at_once_even_once_back_where_it_was() {
    // Arrange -- past the grace, the frame was seen landed.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(false);
    place(&mut table, &mut probe, frame(120, true));
    probe.now += ASYNC_GRACE;
    probe.window().at = at(120);
    let moved = posts_behind(&mut table, &probe);

    // Act -- then the window was sent back.
    probe.window().at = START;
    let back = posts_behind(&mut table, &probe);

    // Assert
    assert!(!moved && !back);
}

#[test]
fn a_change_made_after_a_change_that_was_posted_is_posted_too() {
    // Arrange -- a placement was posted, with no frame before it.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    table.posted(wid(1), &probe, probe.now);

    // Act
    probe.wait(1_000);
    let next = posts_behind(&mut table, &probe);

    // Assert
    assert!(next);
}

#[test]
fn a_window_that_is_not_responding_is_changed_at_once_when_nothing_was_posted() {
    // Arrange
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    probe.window().responding = false;

    // Act / Assert -- as every change was before transitions.
    assert!(!posts_behind(&mut table, &probe));
}

#[test]
fn what_was_posted_to_a_window_with_another_owner_holds_nothing_back() {
    // Arrange -- window 1 closed with a change posted, and a new window
    // was given its id.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    table.posted(wid(1), &probe, probe.now);
    probe.window().owner = 2;

    // Act
    let new_window = posts_behind(&mut table, &probe);

    // Assert
    assert!(!new_window);
    assert!(table.windows.is_empty(), "{:?}", table.windows);
}
