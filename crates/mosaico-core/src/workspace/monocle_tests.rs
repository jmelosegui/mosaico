use super::*;
use crate::id::{wid, wids};

/// A workspace with `windows`, in monocle with `monocle`.
fn in_monocle(windows: &[u64], monocle: u64) -> Workspace {
    let mut ws = Workspace::new();
    for &window in windows {
        ws.add(wid(window));
    }
    ws.set_monocle(true);
    ws.set_monocle_window(Some(wid(monocle)));
    ws
}

#[test]
fn the_windows_that_wait_are_put_behind_once_in_workspace_order() {
    // Arrange
    let mut ws = in_monocle(&[1, 2, 3, 4], 1);
    ws.wait_behind(wid(4));
    ws.wait_behind(wid(2));

    // Act
    let first = ws.take_waiting(wid(1));
    let second = ws.take_waiting(wid(1));

    // Assert
    assert_eq!(first, wids(&[2, 4]));
    assert!(second.is_empty());
}

#[test]
fn a_window_that_leaves_the_workspace_leaves_nothing_waiting() {
    // Arrange -- window 2 waits, leaves, and comes back.
    let mut ws = in_monocle(&[1, 2], 1);
    ws.wait_behind(wid(2));
    ws.remove(wid(2));
    ws.add(wid(2));

    // Act / Assert
    assert!(!ws.waits_behind(wid(2)));
    assert!(ws.take_waiting(wid(1)).is_empty());
}

#[test]
fn only_a_window_on_the_workspace_waits() {
    // Arrange
    let mut ws = in_monocle(&[1], 1);

    // Act
    ws.wait_behind(wid(2));
    ws.add(wid(2));

    // Assert
    assert!(ws.take_waiting(wid(1)).is_empty());
}

#[test]
fn leaving_monocle_forgets_the_windows_that_wait() {
    // Arrange
    let mut ws = in_monocle(&[1, 2], 1);
    ws.wait_behind(wid(2));

    // Act
    ws.set_monocle(false);
    ws.set_monocle(true);

    // Assert
    assert!(ws.take_waiting(wid(1)).is_empty());
}

#[test]
fn the_window_put_behind_last_takes_monocle_on_when_the_monocle_window_leaves() {
    // Arrange -- windows 4 and 5 were put behind window 2, 5 last, so 5
    // is in front of 4.
    let mut ws = in_monocle(&[2, 4, 5], 2);
    ws.wait_behind(wid(4));
    ws.wait_behind(wid(5));
    ws.take_waiting(wid(2));
    ws.remove(wid(2));

    // Act
    let passed = ws.pass_monocle_on();

    // Assert -- 4 is behind it already, and waits for nothing.
    assert!(passed);
    assert_eq!(ws.monocle_window(), Some(wid(5)));
    assert!(ws.take_waiting(wid(5)).is_empty());
}

#[test]
fn a_window_that_still_waits_takes_monocle_on_when_none_was_put_behind() {
    // Arrange -- the workspace was not laid out since windows 4 and 5
    // came, so neither was put behind window 2 yet.
    let mut ws = in_monocle(&[2, 4, 5], 2);
    ws.wait_behind(wid(5));
    ws.wait_behind(wid(4));
    ws.remove(wid(2));

    // Act
    let passed = ws.pass_monocle_on();

    // Assert -- the first in workspace order, and the other still waits
    // behind it.
    assert!(passed);
    assert_eq!(ws.monocle_window(), Some(wid(4)));
    assert_eq!(ws.take_waiting(wid(4)), wids(&[5]));
}

#[test]
fn monocle_is_not_passed_on_when_nothing_waits() {
    // Arrange -- window 3 is on the workspace, but never waited.
    let mut ws = in_monocle(&[2, 3], 2);
    ws.remove(wid(2));

    // Act / Assert
    assert!(!ws.pass_monocle_on());
    assert_eq!(ws.monocle_window(), Some(wid(2)));
}

#[test]
fn the_windows_put_behind_before_wait_behind_the_window_handed_monocle() {
    // Arrange -- window 4 was put behind window 2, which then left, and
    // window 5 waits.
    let mut ws = in_monocle(&[2, 4, 5], 2);
    ws.wait_behind(wid(4));
    ws.take_waiting(wid(2));
    ws.wait_behind(wid(5));
    ws.remove(wid(2));

    // Act
    ws.hand_monocle_to(wid(5));

    // Assert -- window 4 may be in front of window 5, so it goes behind it.
    assert_eq!(ws.monocle_window(), Some(wid(5)));
    assert_eq!(ws.take_waiting(wid(5)), wids(&[4]));
}
