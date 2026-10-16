use std::time::Instant;

use crate::frames::FrameOutcome;
use crate::id::wid;
use crate::motion::test_support::ms;

use super::super::tests::{framed, handled, quick};
use super::super::{Ending, EndingLine, Journal, StepDoneLine};

/// The step's line among `lines`, if it is there.
fn step_line(lines: &[EndingLine]) -> Option<StepDoneLine> {
    lines.iter().find_map(|line| match line {
        EndingLine::Step(line) => Some(line.clone()),
        EndingLine::Motion(_) => None,
    })
}

#[test]
fn a_step_is_done_once_its_last_motion_has_ended_with_its_totals_and_slowest_window() {
    // Arrange -- a swap: window 1 handles three frames, window 2 one, and
    // is found pending at two.
    let t0 = Instant::now();
    let mut journal = Journal::default();
    journal.begin_step();
    journal.moved(wid(1), t0, quick());
    journal.moved(wid(2), t0, quick());
    framed(&mut journal, &[(1, handled()), (2, handled())], t0 + ms(16));
    for i in 2..=3 {
        let outcomes = [(1, handled()), (2, FrameOutcome::Pending)];
        framed(&mut journal, &outcomes, t0 + ms(16 * i));
    }

    // Act
    let first = journal.ended(wid(1), Ending::AtTarget, None, t0 + ms(180));
    let last = journal.ended(wid(2), Ending::AtTarget, None, t0 + ms(180));

    // Assert
    assert_eq!(step_line(&first), None, "window 2 is still moving");
    let done = StepDoneLine {
        windows: vec![wid(1), wid(2)],
        sent: 6,
        handled: 4,
        pending: 2,
        slowest: Some((wid(2), 2)),
    };
    assert_eq!(step_line(&last), Some(done));
}

#[test]
fn a_window_a_later_step_turns_counts_in_that_step() {
    // Arrange -- windows 1 and 2 swap, then a second step turns window 2.
    let t0 = Instant::now();
    let mut journal = Journal::default();
    journal.begin_step();
    journal.moved(wid(1), t0, quick());
    journal.moved(wid(2), t0, quick());
    journal.begin_step();
    journal.moved(wid(2), t0 + ms(50), quick());

    // Act
    let first = journal.ended(wid(1), Ending::AtTarget, None, t0 + ms(180));
    let second = journal.ended(wid(2), Ending::AtTarget, None, t0 + ms(230));

    // Assert -- neither has frames pending, so neither has a slowest.
    let windows = |lines: &[EndingLine]| step_line(lines).map(|line| line.windows);
    assert_eq!(windows(&first), Some(vec![wid(1)]));
    assert_eq!(windows(&second), Some(vec![wid(2)]));
    assert_eq!(step_line(&second).and_then(|line| line.slowest), None);
}

#[test]
fn a_step_whose_last_motion_a_later_step_turns_is_done_with_the_next_step_line() {
    // Arrange -- windows 1 and 2 swap, and window 1 is closed on its way.
    let t0 = Instant::now();
    let mut journal = Journal::default();
    journal.begin_step();
    journal.moved(wid(1), t0, quick());
    journal.moved(wid(2), t0, quick());
    framed(&mut journal, &[(1, handled()), (2, handled())], t0 + ms(16));
    let gone = journal.ended(wid(1), Ending::Destroyed, None, t0 + ms(40));

    // Act -- the retile turns window 2, and its motion then ends.
    journal.begin_step();
    journal.moved(wid(2), t0 + ms(50), quick());
    let closed = journal.closed_steps();
    let last = journal.ended(wid(2), Ending::AtTarget, None, t0 + ms(230));

    // Assert
    assert_eq!(step_line(&gone), None, "window 2 is still moving");
    let swap = StepDoneLine {
        windows: vec![wid(1)],
        sent: 1,
        handled: 1,
        pending: 0,
        slowest: None,
    };
    assert_eq!(closed, vec![swap]);
    assert_eq!(
        step_line(&last).map(|line| line.windows),
        Some(vec![wid(2)])
    );
    assert_eq!(journal.tallies.len(), 0, "every step is done");
    assert_eq!(
        journal.closed_steps(),
        Vec::new(),
        "each line is given once"
    );
}
