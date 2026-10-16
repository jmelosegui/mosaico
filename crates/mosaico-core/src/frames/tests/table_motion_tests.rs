//! What the frame table does at the start of a motion, and with windows
//! that are gone, have another owner, are minimized or maximized, or do
//! not respond.

use super::*;

#[test]
fn what_the_first_frame_of_a_motion_reads_is_used_until_the_next_motion() {
    // Arrange -- the window is read with 10 to keep this time.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    place(&mut table, &mut probe, frame(120, true));
    probe.offset = 10;

    // Act
    place(&mut table, &mut probe, frame(140, false));
    place(&mut table, &mut probe, frame(160, true));
    probe.offset = 20;
    place(&mut table, &mut probe, frame(180, false));

    // Assert
    let posted: Vec<i32> = probe.posted.iter().map(|(_, to)| to.x).collect();
    assert_eq!(posted, vec![120, 140, 170, 190]);
    assert_eq!(probe.keeps.get(), 2);
}

#[test]
fn a_new_motion_is_held_back_by_a_frame_of_the_last_one_still_on_its_way() {
    // Arrange
    let mut table = FrameTable::default();
    let mut probe = Probe::new(false);
    place(&mut table, &mut probe, frame(120, true));

    // Act
    let new_motion = place(&mut table, &mut probe, frame(160, true));

    // Assert
    assert_eq!(new_motion, FrameOutcome::Pending);
    assert_eq!(probe.posted, vec![(wid(1), at(120))]);
}

#[test]
fn a_window_with_another_owner_starts_over_with_the_first_frame_of_its_motion() {
    // Arrange -- a frame is queued for window 1, which closes, and a new
    // window is given its id, where it was.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(false);
    place(&mut table, &mut probe, frame(120, true));
    probe.window().owner = 2;

    // Act
    let outcome = place(&mut table, &mut probe, frame(140, true));

    // Assert
    assert_eq!(outcome, FrameOutcome::Handled { reached: START });
    assert_eq!(probe.posted, vec![(wid(1), at(120)), (wid(1), at(140))]);
}

#[test]
fn a_window_with_another_owner_is_gone_for_the_rest_of_the_old_motion() {
    // Arrange
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    place(&mut table, &mut probe, frame(120, true));
    probe.window().owner = 2;

    // Act
    let outcome = place(&mut table, &mut probe, frame(140, false));

    // Assert -- nothing of the old motion goes to the new window.
    assert_eq!(outcome, FrameOutcome::Gone);
    assert_eq!(probe.posted, vec![(wid(1), at(120))]);
}

#[test]
fn a_window_that_is_gone_is_dropped_from_the_table() {
    // Arrange -- window 2 got a frame, then closed, and gets no more.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(false);
    let second = Window { ..*probe.window() };
    probe.windows.insert(wid(2), second);
    let both = [
        frame(120, true),
        Frame {
            window: wid(2),
            ..frame(120, true)
        },
    ];
    let now = probe.now;
    table.place(&both, &mut probe, now, &mut Vec::new());
    probe.windows.clear();

    // Act
    let outcome = place(&mut table, &mut probe, frame(140, false));

    // Assert
    assert_eq!(outcome, FrameOutcome::Gone);
    assert!(table.windows.is_empty(), "{:?}", table.windows);
}

#[test]
fn minimized_and_maximized_windows_are_left_alone() {
    // Arrange
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);

    // Act
    probe.window().minimized = true;
    let minimized = place(&mut table, &mut probe, frame(120, true));
    probe.window().minimized = false;
    probe.window().maximized = true;
    let maximized = place(&mut table, &mut probe, frame(120, true));

    // Assert
    assert_eq!(minimized, FrameOutcome::Minimized);
    assert_eq!(maximized, FrameOutcome::Maximized);
    assert!(probe.posted.is_empty(), "{:?}", probe.posted);
}

#[test]
fn a_window_that_is_not_responding_is_pending_and_gets_nothing() {
    // Arrange
    let mut table = FrameTable::default();
    let mut probe = Probe::new(true);
    probe.window().responding = false;

    // Act
    let outcome = place(&mut table, &mut probe, frame(120, true));
    probe.wait(limit_ms() * 4);
    let later = place(&mut table, &mut probe, frame(140, false));

    // Assert
    assert_eq!([outcome, later], [FrameOutcome::Pending; 2]);
    assert!(probe.posted.is_empty(), "{:?}", probe.posted);
}
