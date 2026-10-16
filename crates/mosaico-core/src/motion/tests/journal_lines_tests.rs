use crate::id::wid;
use crate::motion::test_support::spec;
use crate::motion::{ApplySnap, Easing};

use super::*;

/// A step line for the windows that start and those that turn.
fn step(started: &[u64], turned: &[u64]) -> StepLine {
    StepLine {
        started: started.iter().map(|&id| wid(id)).collect(),
        turned: turned.iter().map(|&id| wid(id)).collect(),
        spec: spec(180, Easing::EaseOutCubic),
    }
}

#[test]
fn a_step_line_names_the_easing_only_for_the_windows_that_start() {
    // Arrange
    let lines = [
        step(&[0x1A, 0x2B], &[]),
        step(&[], &[0x3C]),
        step(&[0x1A], &[0x3C]),
    ];

    // Act
    let texts = lines.map(|line| line.to_string());

    // Assert -- a window that turns keeps its speed, along a curve of its
    // own.
    assert_eq!(
        texts,
        [
            "transition: 2 window(s) move over 180 ms with ease-out-cubic: 0x1A, 0x2B",
            "transition: 1 window(s) turn to new targets over 180 ms: 0x3C",
            "transition: 1 window(s) move over 180 ms with ease-out-cubic: 0x1A; 1 window(s) \
             turn to new targets over 180 ms: 0x3C",
        ]
    );
}

#[test]
fn snap_lines_name_the_window_or_count_the_placements_and_say_why() {
    // Arrange
    let window = SnapLine {
        window: wid(0x1A),
        why: ApplySnap::ShortTravel(3),
    };
    let step = StepSnapLine {
        placements: 4,
        why: StepSnap::Rebuild,
    };

    // Act / Assert
    assert_eq!(
        window.to_string(),
        "transition: 0x1A snaps: it travels 3 px, 4 or less"
    );
    assert_eq!(
        step.to_string(),
        "transition: 4 placement(s) snap: the step lays windows out from scratch"
    );
}
