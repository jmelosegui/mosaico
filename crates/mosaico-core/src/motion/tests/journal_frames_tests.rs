use std::time::Instant;

use crate::frames::FrameOutcome;
use crate::id::wid;
use crate::motion::test_support::ms;

use super::super::tests::{framed, handled, motion_line, quick};
use super::super::{BehindLine, Ending, Journal, Stop};

#[test]
fn a_window_found_gone_maximized_or_minimized_ends_its_motion_there() {
    // Arrange
    let t0 = Instant::now();
    let mut journal = Journal::default();
    for id in [1, 2, 3] {
        journal.moved(wid(id), t0, quick());
    }

    // Act
    let outcomes = [
        (1, FrameOutcome::Gone),
        (2, FrameOutcome::Maximized),
        (3, FrameOutcome::Minimized),
    ];
    let lines = framed(&mut journal, &outcomes, t0 + ms(16));

    // Assert -- each after its one frame, posted.
    let ended = |id| motion_line(&lines, wid(id)).map(|l| (l.ending, l.took, l.sent));
    let stopped = |stop| Some((Ending::Stopped(stop), ms(16), 1));
    assert_eq!(ended(1), Some((Ending::Destroyed, ms(16), 1)));
    assert_eq!(ended(2), stopped(Stop::Maximized));
    assert_eq!(ended(3), stopped(Stop::Minimized));
}

#[test]
fn a_window_that_fell_behind_did_so_at_the_last_frame_posted_for_it() {
    // Arrange -- window 2 has one frame handled, then three found pending.
    let t0 = Instant::now();
    let mut journal = Journal::default();
    journal.moved(wid(1), t0, quick());
    journal.moved(wid(2), t0, quick());
    framed(&mut journal, &[(1, handled()), (2, handled())], t0 + ms(16));
    for i in 2..=4 {
        let outcomes = [(1, handled()), (2, FrameOutcome::Pending)];
        framed(&mut journal, &outcomes, t0 + ms(16 * i));
    }

    // Act -- as the frame step says it fell behind.
    let line = journal.fell_behind(wid(2));

    // Assert
    let behind = BehindLine {
        window: wid(2),
        frame: 4,
    };
    assert_eq!(line, Some(behind));
    assert_eq!(journal.fell_behind(wid(7)), None, "no motion of it");
}

#[test]
fn frames_for_windows_it_has_no_motion_of_are_left_out() {
    // Arrange -- no motion of window 7.
    let t0 = Instant::now();
    let mut journal = Journal::default();

    // Act
    let lines = framed(&mut journal, &[(7, FrameOutcome::Gone)], t0);

    // Assert
    assert!(lines.is_empty(), "{lines:?}");
}
