//! What the log is told of transitions, as the tiling manager runs them.

use crate::Action;

use super::{animated, frame_by_frame, settle, step, swap, two};

#[path = "motion_log_end_tests.rs"]
mod end_tests;
#[path = "motion_log_frames_tests.rs"]
mod frames_tests;
#[path = "motion_log_landing_tests.rs"]
mod landing_tests;
#[path = "motion_log_placed_tests.rs"]
mod placed_tests;
#[path = "motion_log_snap_tests.rs"]
mod snap_tests;
#[path = "motion_log_step_tests.rs"]
mod step_tests;
#[path = "motion_log_straight_tests.rs"]
mod straight_tests;

/// How many of `lines` start with `start`.
fn starting(lines: &[String], start: &str) -> usize {
    lines.iter().filter(|line| line.starts_with(start)).count()
}

#[test]
fn a_swap_is_logged_as_one_step_whose_motions_end_at_their_targets() {
    // Arrange
    let (platform, mut manager) = two(&animated());

    // Act
    swap(&mut manager);
    settle(&platform, &mut manager);

    // Assert -- the step, each motion as it ends, its placement posted
    // behind its frames and finding the window there already, then the
    // step's end: no line for any frame on its own, nor for a landing
    // that was not waited for.
    let lines = manager.transition_lines();
    assert_eq!(lines.len(), 4, "{lines:#?}");
    let step = "transition: 2 window(s) move over 180 ms with ease-out-cubic: ";
    assert!(lines[0].starts_with(step), "{lines:#?}");
    for id in ["0x1", "0x2"] {
        let ended = format!("transition: {id} ended at its target after ");
        assert_eq!(starting(&lines, &ended), 1, "{lines:#?}");
    }
    let posted = "; its placement posted behind what was on its way, the window there already";
    assert!(lines[1..3].iter().all(|line| line.ends_with(posted)));
    let done = "transition: a step of 2 window(s) is done: ";
    assert!(lines[3].starts_with(done), "{lines:#?}");
    assert!(lines[3].ends_with("none found pending"), "{lines:#?}");
}

#[test]
fn a_step_that_falls_behind_is_logged_with_the_window_and_its_frame() {
    // Arrange -- window 2's application stops responding as they swap.
    let (platform, mut manager) = two(&animated());
    platform.edit(2, |w| w.hung = true);

    // Act
    swap(&mut manager);
    frame_by_frame(&platform, &mut manager);

    // Assert -- found pending at its first four frames, 64 ms without being
    // seen to move, it ends both motions, and is the slowest of the step.
    let lines = manager.transition_lines();
    let behind = "transition: 0x2 fell behind at its frame 4: its step goes straight to its \
                  targets";
    assert_eq!(starting(&lines, behind), 1, "{lines:#?}");
    for id in ["0x1", "0x2"] {
        let ended = format!("transition: {id} went straight to its target, its step fell behind");
        assert_eq!(starting(&lines, &ended), 1, "{lines:#?}");
    }
    let done = "transition: a step of 2 window(s) is done: ";
    let done = lines.iter().find(|line| line.starts_with(done));
    let slowest = done.is_some_and(|line| line.ends_with("slowest 0x2, 4 pending"));
    assert!(slowest, "{lines:#?}");
}

#[test]
fn a_window_minimized_on_its_way_is_logged_as_stopped_there() {
    // Arrange -- window 1 is focused, and one frame on its way right.
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    step(&platform, &mut manager);

    // Act
    manager.handle_action(&Action::MinimizeFocused);
    settle(&platform, &mut manager);

    // Assert
    let lines = manager.transition_lines();
    let stopped = "transition: 0x1 stopped where it was, minimized after ";
    assert_eq!(starting(&lines, stopped), 1, "{lines:#?}");
    let hidden = "transition: 0x1 stopped where it was, hidden";
    assert_eq!(starting(&lines, hidden), 0, "{lines:#?}");
}

#[test]
fn a_retile_that_repeats_the_targets_starts_no_step_and_ends_each_motion_once() {
    // Arrange -- a swap, one frame on its way.
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    step(&platform, &mut manager);

    // Act
    manager.handle_action(&Action::Retile);
    manager.flush_pending();
    settle(&platform, &mut manager);

    // Assert -- the windows go on to the targets they had.
    let lines = manager.transition_lines();
    assert_eq!(
        starting(&lines, "transition: 2 window(s) move"),
        1,
        "{lines:#?}"
    );
    let turns = lines
        .iter()
        .filter(|line| line.contains(" turn to new targets "));
    assert_eq!(turns.count(), 0, "{lines:#?}");
    for id in ["0x1", "0x2"] {
        let ended = format!("transition: {id} ended at its target after ");
        assert_eq!(starting(&lines, &ended), 1, "{lines:#?}");
    }
}
