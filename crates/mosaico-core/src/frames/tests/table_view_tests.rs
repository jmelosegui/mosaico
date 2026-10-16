//! What the table keeps for the changes a platform posts behind a
//! window's frames: what it read with them, and whether the window is to
//! be in view once what was posted lands.

use std::time::Duration;

use super::*;

/// Whether window 1 is to be in view once what was posted lands.
fn in_view(table: &mut FrameTable<i32>, probe: &Probe) -> Option<bool> {
    table.in_view_once_landed(wid(1), probe)
}

#[test]
fn what_was_read_with_the_frames_is_kept_for_the_window_while_its_owner_has_it() {
    // Arrange
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    probe.offset = 7;
    let before = table.kept_for(wid(1), &probe);
    place(&mut table, &mut probe, frame(120, true));

    // Act
    let kept = table.kept_for(wid(1), &probe);
    probe.window().owner = 2;
    let new_window = table.kept_for(wid(1), &probe);

    // Assert
    assert_eq!((before, kept, new_window), (None, Some(7), None));
}

#[test]
fn a_change_of_show_state_posted_says_how_the_window_is_to_be_until_seen_so() {
    // Arrange
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    let before = in_view(&mut table, &probe);

    // Act -- a hide, which lands, then a show, which lands.
    table.posted_show_state(wid(1), &probe, probe.now, false);
    let hiding = in_view(&mut table, &probe);
    probe.window().shown = false;
    let hidden = in_view(&mut table, &probe);
    table.posted_show_state(wid(1), &probe, probe.now, true);
    let showing = in_view(&mut table, &probe);
    probe.window().shown = true;
    let shown = in_view(&mut table, &probe);

    // Assert -- seen as it was to be, the window says so itself.
    assert_eq!([before, hiding, hidden], [None, Some(false), None]);
    assert_eq!([showing, shown], [Some(true), None]);
}

#[test]
fn a_hide_still_queued_keeps_the_window_out_of_view_however_long_it_takes() {
    // Arrange -- the application does not get to it.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    table.posted_show_state(wid(1), &probe, probe.now, false);

    // Act
    probe.now += Duration::from_secs(500);
    let later = in_view(&mut table, &probe);
    probe.window().owner = 2;
    let new_window = in_view(&mut table, &probe);

    // Assert
    assert_eq!((later, new_window), (Some(false), None));
}

#[test]
fn a_window_seen_hidden_by_posts_behind_is_left_to_its_own_state_after() {
    // Arrange -- the hide landed, and a change was made after it.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    table.posted_show_state(wid(1), &probe, probe.now, false);
    probe.window().shown = false;
    table.posts_behind(wid(1), &probe, probe.now);

    // Act -- then the application showed it itself.
    probe.window().shown = true;
    let shown = in_view(&mut table, &probe);

    // Assert
    assert_eq!(shown, None);
}

#[test]
fn a_change_of_show_state_made_at_once_leaves_the_window_as_it_makes_it() {
    // Arrange -- a hide was posted, then a show made at once.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    table.posted_show_state(wid(1), &probe, probe.now, false);
    probe.now += ASYNC_GRACE;

    // Act
    table.show_state_made(wid(1), &probe);

    // Assert
    assert_eq!(in_view(&mut table, &probe), None);
}

#[test]
fn a_change_of_show_state_made_at_once_keeps_nothing_for_a_window_with_nothing_posted() {
    // Arrange
    let mut table = FrameTable::default();
    let probe = Probe::new(true);

    // Act
    table.show_state_made(wid(1), &probe);

    // Assert
    assert!(table.windows.is_empty(), "{:?}", table.windows);
}
