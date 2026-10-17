//! The frame step: on display frames, and when a motion is due to end.

use crate::tiler::QUIET;

use super::*;

fn frame() -> Message<Task> {
    Message::Frame
}

#[test]
fn the_frame_step_runs_once_after_the_tasks_and_events_of_its_wake() {
    // Arrange -- the loop was busy, and two frames queued up.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let batch = received(vec![frame(), task("hotkey"), event(focused(1)), frame()]);

    // Act
    let steps = scheduler.step(batch, t0);

    // Assert
    assert_eq!(
        steps,
        vec![
            Step::Run("hotkey"),
            Step::Events(vec![focused(1)]),
            Step::Frame,
        ]
    );
}

#[test]
fn a_frame_does_not_start_the_quiet_window_over() {
    // Arrange -- a focus change is held at t0.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);
    scheduler.begin_wait(held, None, None, t0);

    // Act -- a frame comes 10 ms into the quiet window.
    let steps = scheduler.step(received(vec![frame()]), t0 + ms(10));
    let wait = scheduler.begin_wait(held, None, None, t0 + ms(11));

    // Assert
    assert_eq!(steps, vec![Step::Frame]);
    assert_eq!(wait, Some(QUIET - ms(11)));
}

#[test]
fn a_frame_finds_what_is_due_by_the_time_it_arrives() {
    // Arrange -- the loop gets to the frame after the quiet window.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.begin_wait(Some(t0), None, None, t0);

    // Act
    let steps = scheduler.step(received(vec![frame()]), t0 + ms(30));

    // Assert -- the flush first, so what it starts moving takes a step.
    assert_eq!(steps, vec![Step::Flush, Step::Frame]);
}

#[test]
fn a_display_change_that_settles_while_frames_arrive_is_rebuilt_for() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.step(display_changed(), t0);
    scheduler.begin_wait(None, None, None, t0 + SETTLE - ms(5));

    // Act -- frames are all that arrives once the change has settled.
    let steps = scheduler.step(received(vec![frame()]), t0 + SETTLE + ms(1));

    // Assert
    assert_eq!(steps, vec![Step::RebuildMonitors, Step::Frame]);
}
