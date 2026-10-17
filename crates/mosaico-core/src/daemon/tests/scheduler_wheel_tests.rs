//! The wheel step: once the wheel's turns on a bar are due, at its place
//! in the wake.

use crate::tiler::QUIET;

use super::*;

#[test]
fn the_wait_ends_with_a_wheel_step_when_the_turns_are_due() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();

    // Act
    let wait = scheduler.begin_wait(None, None, Some(t0 + ms(120)), t0);
    let steps = scheduler.step(timed_out(), t0 + ms(120));

    // Assert
    assert_eq!(wait, Some(ms(120)));
    assert_eq!(steps, vec![Step::Wheel]);
}

#[test]
fn turns_due_while_the_loop_was_busy_end_the_wait_at_once() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();

    // Act
    let wait = scheduler.begin_wait(None, None, Some(t0), t0 + ms(30));

    // Assert
    assert_eq!(wait, Some(Duration::ZERO));
}

#[test]
fn the_wheel_step_comes_after_the_tasks_and_events_and_before_the_frame() {
    // Arrange -- a click, a focus change and a frame came as the turns
    // fell due.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.begin_wait(None, None, Some(t0 + ms(5)), t0);
    let batch = received(vec![Message::Frame, task("click"), event(focused(1))]);

    // Act
    let steps = scheduler.step(batch, t0 + ms(5));

    // Assert
    assert_eq!(
        steps,
        vec![
            Step::Run("click"),
            Step::Events(vec![focused(1)]),
            Step::Wheel,
            Step::Frame,
        ]
    );
}

#[test]
fn a_wake_before_the_turns_are_due_takes_no_wheel_step() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.begin_wait(None, None, Some(t0 + ms(120)), t0);

    // Act -- another turn comes, which the bar adds to the others.
    let steps = scheduler.step(received(vec![task("turn")]), t0 + ms(50));

    // Assert
    assert_eq!(steps, vec![Step::Run("turn")]);
}

#[test]
fn a_flush_due_in_the_same_wake_comes_before_the_wheel_step() {
    // Arrange -- a focus change is held at t0, and the turns fall due as
    // the quiet window ends.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.begin_wait(Some(t0), None, Some(t0 + QUIET), t0);

    // Act
    let steps = scheduler.step(timed_out(), t0 + QUIET);

    // Assert
    assert_eq!(steps, vec![Step::Flush, Step::Wheel]);
}

#[test]
fn the_wheel_step_ends_the_wait_as_input_does() {
    // Arrange -- the switch the wheel makes holds a focus change, and
    // takes 30 ms.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.begin_wait(None, None, Some(t0 + ms(10)), t0);
    scheduler.step(timed_out(), t0 + ms(10));

    // Act
    let wait = scheduler.begin_wait(Some(t0 + ms(20)), None, None, t0 + ms(40));

    // Assert -- the time the switch took does not count as quiet.
    assert_eq!(wait, Some(QUIET));
}
