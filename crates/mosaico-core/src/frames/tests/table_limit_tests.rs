//! How long a frame posted to a window holds the next one back: while
//! the window has not moved, for the limit at most.

use super::*;

#[test]
fn a_frame_holds_the_next_one_back_while_the_window_has_not_moved() {
    // Arrange
    let mut table = FrameTable::default();
    let mut probe = Probe::new(false);
    place(&mut table, &mut probe, frame(120, true));

    // Act -- until just short of the limit.
    let at_once = place(&mut table, &mut probe, frame(140, false));
    probe.wait(limit_ms() - 1);
    let later = place(&mut table, &mut probe, frame(160, false));

    // Assert
    assert_eq!([at_once, later], [FrameOutcome::Pending; 2]);
    assert_eq!(probe.posted, vec![(wid(1), at(120))]);
}

#[test]
fn a_slow_window_gets_one_frame_per_limit() {
    // Arrange -- its application gets to none of them.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(false);
    place(&mut table, &mut probe, frame(120, true));

    // Act -- a frame every 50 ms for two limits.
    let mut handled = 0;
    for step in 1..=(2 * limit_ms() / 50) {
        probe.wait(50);
        let found = place(&mut table, &mut probe, frame(120 + step as i32, false));
        handled += usize::from(found != FrameOutcome::Pending);
    }

    // Assert -- one more at each limit.
    assert_eq!(handled, 2);
    assert_eq!(probe.posted.len(), 3);
}

#[test]
fn a_window_that_moved_gets_the_next_frame_at_once() {
    // Arrange -- it rounded the frame, or was moved elsewhere.
    let mut table = FrameTable::default();
    let mut probe = Probe::new(false);
    place(&mut table, &mut probe, frame(120, true));
    probe.window().at = at(118);

    // Act
    let outcome = place(&mut table, &mut probe, frame(140, false));

    // Assert
    assert_eq!(outcome, FrameOutcome::Handled { reached: at(118) });
    assert_eq!(probe.posted, vec![(wid(1), at(120)), (wid(1), at(140))]);
}
