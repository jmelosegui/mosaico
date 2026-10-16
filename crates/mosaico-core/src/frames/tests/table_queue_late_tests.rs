//! Whether the table still has a window's changes posted once the grace
//! has run out: while the window has not been seen to move since the last
//! thing posted to it, whatever that was, and while its application does
//! not respond.

use super::*;

/// Whether a change made to window 1 now is posted behind.
fn posts_behind(table: &mut FrameTable<i32>, probe: &Probe) -> bool {
    table.posts_behind(wid(1), probe, probe.now)
}

#[test]
fn a_placement_posted_after_the_last_frame_landed_holds_until_the_window_moves() {
    // Arrange -- the frame landed, and the motion's placement was posted.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    place(&mut table, &mut probe, frame(120, true));
    table.posted(wid(1), &probe, probe.now);

    // Act
    probe.now += ASYNC_GRACE * 4;
    let queued = posts_behind(&mut table, &probe);
    probe.window().at = at(300);
    let landed = posts_behind(&mut table, &probe);

    // Assert -- no timer alone ends it.
    assert!(queued);
    assert!(!landed);
}

#[test]
fn a_change_of_show_state_posted_holds_until_the_window_moves() {
    // Arrange -- a hide, which does not move the window.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    table.posted_show_state(wid(1), &probe, probe.now, false);

    // Act
    probe.now += ASYNC_GRACE * 4;
    let later = posts_behind(&mut table, &probe);

    // Assert
    assert!(later);
}

#[test]
fn the_later_of_two_queued_frames_holds_past_the_grace_only_while_the_application_hangs() {
    // Arrange -- a slow window got a second frame after the limit, and
    // took the first one late.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(false);
    place(&mut table, &mut probe, frame(120, true));
    probe.wait(limit_ms());
    place(&mut table, &mut probe, frame(140, false));
    probe.window().at = at(120);
    probe.now += ASYNC_GRACE;

    // Act
    let responding = posts_behind(&mut table, &probe);
    probe.window().responding = false;
    let hung = posts_behind(&mut table, &probe);

    // Assert -- the gap the contract names, and how a hang closes it.
    assert!(!responding);
    assert!(hung);
}

#[test]
fn a_move_made_by_anything_else_ends_the_order_past_the_grace() {
    // Arrange -- a placement was posted, then the user moved the window.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    place(&mut table, &mut probe, frame(120, true));
    table.posted(wid(1), &probe, probe.now);
    probe.window().at = at(500);

    // Act
    probe.now += ASYNC_GRACE;
    let moved = posts_behind(&mut table, &probe);
    probe.window().responding = false;
    let hung = posts_behind(&mut table, &probe);
    probe.window().responding = true;
    let back = posts_behind(&mut table, &probe);

    // Assert -- an application that stops responding turns it back on.
    assert!(!moved && hung && !back);
}

#[test]
fn a_window_given_frames_while_it_hangs_gets_its_changes_posted() {
    // Arrange -- its frames stayed pending, so nothing was posted.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    probe.window().responding = false;
    let pending = place(&mut table, &mut probe, frame(120, true));

    // Act
    let hung = posts_behind(&mut table, &probe);
    probe.window().responding = true;
    let responding = posts_behind(&mut table, &probe);

    // Assert -- a change made at once would wait for it.
    assert_eq!(pending, FrameOutcome::Pending);
    assert!(hung);
    assert!(!responding);
}
