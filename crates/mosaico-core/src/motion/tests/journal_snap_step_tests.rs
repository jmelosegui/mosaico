//! The line of a step that snaps as a whole.

use crate::motion::{StepFacts, StepSnap};

use super::super::super::StepSnapLine;
use super::step;

#[test]
fn a_step_that_snaps_as_a_whole_gets_one_line_with_the_reason_its_placements_get() {
    // Arrange -- a rebuild, and a step that moves 17 windows, whatever the
    // window the facts are of.
    let rebuild = StepFacts {
        rebuild: true,
        appears: true,
        ..step()
    };
    let crowded = StepFacts {
        windows_moved: 17,
        same_monitor: false,
        ..step()
    };

    // Act
    let lines = [
        StepSnapLine::of_step(&rebuild, 3),
        StepSnapLine::of_step(&crowded, 17),
    ];

    // Assert
    let line = |placements, why| Some(StepSnapLine { placements, why });
    let expected = [
        line(3, StepSnap::Rebuild),
        line(17, StepSnap::TooManyWindows(17)),
    ];
    assert_eq!(lines, expected);
}

#[test]
fn a_step_that_may_animate_has_no_placements_or_has_transitions_off_gets_no_step_line() {
    // Arrange
    let off = StepFacts {
        enabled: false,
        ..step()
    };
    let rebuild = StepFacts {
        rebuild: true,
        ..step()
    };

    // Act
    let lines = [
        StepSnapLine::of_step(&step(), 2),
        StepSnapLine::of_step(&off, 2),
        StepSnapLine::of_step(&rebuild, 0),
    ];

    // Assert
    assert_eq!(lines, [None, None, None]);
}
