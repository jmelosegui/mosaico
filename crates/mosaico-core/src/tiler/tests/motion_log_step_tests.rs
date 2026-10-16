//! What the log is told of the steps that set windows moving: each step's
//! line before its first frame, and the line of a step done once the last
//! of its motions has ended, or a later step has turned it.

use crate::id::wid;
use crate::tiler::tests::fake_platform::FakePlatform;
use crate::tiler::tests::manager_support::start;
use crate::{Action, Direction, WindowEvent};

use super::super::{animated, settle, step, swap, two};
use super::starting;

/// Where in `lines` the first that starts with `start` is.
fn first(lines: &[String], start: &str) -> Option<usize> {
    lines.iter().position(|line| line.starts_with(start))
}

#[test]
fn a_step_is_logged_as_its_first_frame_is_taken() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    let before = manager.transition_lines();

    // Act
    step(&platform, &mut manager);

    // Assert
    assert!(before.is_empty(), "{before:#?}");
    let lines = manager.transition_lines();
    let step = "transition: 2 window(s) move over 180 ms with ease-out-cubic: 0x2, 0x1";
    assert_eq!(lines, [step], "{lines:#?}");
}

#[test]
fn a_step_is_logged_before_the_next_one_starts() {
    // Arrange -- three windows; window 1 swaps with the one on its right.
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0), (3, 0)]);
    let mut manager = start(&platform, None, &animated());
    swap(&mut manager);

    // Act -- before the first frame, it moves on down, and turns.
    manager.handle_action(&Action::Move(Direction::Down));
    manager.flush_pending();
    step(&platform, &mut manager);

    // Assert -- each step gets its own line.
    let lines = manager.transition_lines();
    let second = "transition: 1 window(s) move over 180 ms with ease-out-cubic: 0x3; 1 window(s) \
                  turn to new targets over 180 ms: 0x1";
    assert_eq!(lines.len(), 2, "{lines:#?}");
    let first = "transition: 2 window(s) move over 180 ms with ease-out-cubic: 0x2, 0x1";
    assert_eq!(lines[0], first, "{lines:#?}");
    assert_eq!(lines[1], second, "{lines:#?}");
}

#[test]
fn a_step_whose_window_closes_on_its_way_is_done_once_the_retile_turns_the_other() {
    // Arrange -- a swap, one frame on its way.
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    step(&platform, &mut manager);

    // Act -- window 1 closes, and window 2 turns to fill the monitor.
    platform.close_window(1);
    manager.handle_event(&WindowEvent::Destroyed { window: wid(1) });
    settle(&platform, &mut manager);

    // Assert -- the swap is done as the retile turns its last motion.
    let lines = manager.transition_lines();
    let swap = first(&lines, "transition: a step of 1 window(s) is done: 0x1;");
    let turn = first(&lines, "transition: 1 window(s) turn to new targets ");
    assert!(swap.is_some() && swap < turn, "{lines:#?}");
    let retile = "transition: a step of 1 window(s) is done: 0x2;";
    assert_eq!(starting(&lines, retile), 1, "{lines:#?}");
}

#[test]
fn a_step_whose_window_the_user_minimizes_on_its_way_is_done_once_the_retile_turns_the_other() {
    // Arrange
    let (platform, mut manager) = two(&animated());
    swap(&mut manager);
    step(&platform, &mut manager);

    // Act
    platform.edit(2, |w| w.minimized = true);
    manager.handle_event(&WindowEvent::Minimized { window: wid(2) });
    settle(&platform, &mut manager);

    // Assert
    let lines = manager.transition_lines();
    let swap = first(&lines, "transition: a step of 1 window(s) is done: 0x2;");
    let turn = first(&lines, "transition: 1 window(s) turn to new targets ");
    assert!(swap.is_some() && swap < turn, "{lines:#?}");
    let retile = "transition: a step of 1 window(s) is done: 0x1;";
    assert_eq!(starting(&lines, retile), 1, "{lines:#?}");
}
