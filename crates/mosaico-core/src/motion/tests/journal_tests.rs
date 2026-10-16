use std::time::Instant;

use crate::frames::{Frame, FrameOutcome};
use crate::id::wid;
use crate::motion::test_support::{ms, spec};
use crate::motion::{Easing, MotionSpec, StepLine};
use crate::{Rect, WindowId};

use super::*;

const HALF: Rect = Rect {
    x: 400,
    y: 0,
    width: 800,
    height: 900,
};

/// One frame step: a frame for each window, and what was found of it.
/// Returns the lines of the motions it ended.
pub(super) fn framed(
    journal: &mut Journal,
    outcomes: &[(u64, FrameOutcome)],
    now: Instant,
) -> Vec<EndingLine> {
    let frames: Vec<Frame> = outcomes
        .iter()
        .map(|&(id, _)| Frame {
            window: wid(id),
            rect: HALF,
            first: false,
        })
        .collect();
    let found: Vec<FrameOutcome> = outcomes.iter().map(|&(_, outcome)| outcome).collect();
    journal.framed(&frames, &found, now)
}

/// A frame the window had handled the one before for.
pub(super) fn handled() -> FrameOutcome {
    FrameOutcome::Handled { reached: HALF }
}

/// A motion lasting 180 ms.
pub(super) fn quick() -> MotionSpec {
    spec(180, Easing::Linear)
}

/// The line of the motion of `window` among `lines`.
pub(super) fn motion_line(lines: &[EndingLine], window: WindowId) -> Option<EndLine> {
    lines.iter().find_map(|line| match line {
        EndingLine::Motion(line) if line.window == window => Some(*line),
        _ => None,
    })
}

#[test]
fn a_step_line_names_the_windows_that_start_and_those_that_turn_once_each() {
    // Arrange -- windows 1 and 2 start, then window 1 turns and window 3
    // starts.
    let t0 = Instant::now();
    let mut journal = Journal::default();
    for id in [1, 2, 1] {
        journal.moved(wid(id), t0, quick());
    }
    let first = journal.step_line();
    journal.begin_step();

    // Act
    journal.moved(wid(1), t0 + ms(50), quick());
    journal.moved(wid(3), t0 + ms(50), quick());
    let second = journal.step_line();

    // Assert
    let line = |started: &[u64], turned: &[u64]| StepLine {
        started: started.iter().map(|&id| wid(id)).collect(),
        turned: turned.iter().map(|&id| wid(id)).collect(),
        spec: quick(),
    };
    assert_eq!(first, Some(line(&[1, 2], &[])));
    assert_eq!(second, Some(line(&[3], &[1])));
    assert_eq!(journal.step_line(), None);
}

#[test]
fn a_motion_that_runs_out_ends_at_its_target_with_what_its_frames_found() {
    // Arrange -- a frame handled, one found pending, then one more handled.
    let t0 = Instant::now();
    let mut journal = Journal::default();
    journal.moved(wid(1), t0, quick());
    framed(&mut journal, &[(1, handled())], t0 + ms(16));
    framed(&mut journal, &[(1, FrameOutcome::Pending)], t0 + ms(32));
    framed(&mut journal, &[(1, handled())], t0 + ms(48));

    // Act
    let lines = journal.ended(
        wid(1),
        Ending::AtTarget,
        Some(Placed::Landing),
        t0 + ms(200),
    );

    // Assert
    let expected = EndLine {
        window: wid(1),
        ending: Ending::AtTarget,
        took: ms(200),
        planned: ms(180),
        sent: 3,
        handled: 2,
        pending: 1,
        placed: Some(Placed::Landing),
    };
    assert_eq!(motion_line(&lines, wid(1)), Some(expected));
    let again = journal.ended(wid(1), Ending::AtTarget, None, t0 + ms(216));
    assert!(again.is_empty(), "it is forgotten: {again:?}");
}

#[test]
fn a_new_target_keeps_the_start_of_the_motion_and_moves_when_it_is_due() {
    // Arrange
    let t0 = Instant::now();
    let mut journal = Journal::default();
    journal.moved(wid(1), t0, quick());

    // Act
    journal.moved(wid(1), t0 + ms(100), quick());
    let lines = journal.ended(wid(1), Ending::AtTarget, None, t0 + ms(300));

    // Assert
    let times = motion_line(&lines, wid(1)).map(|line| (line.took, line.planned));
    assert_eq!(times, Some((ms(300), ms(280))));
}

#[test]
fn a_window_it_has_no_motion_of_has_no_line_when_it_stops() {
    // Arrange
    let t0 = Instant::now();
    let mut journal = Journal::default();
    journal.moved(wid(1), t0, quick());

    // Act
    let lines = journal.ended(wid(9), Ending::Stopped(Stop::Hidden), None, t0 + ms(50));

    // Assert
    assert!(lines.is_empty(), "{lines:?}");
}
