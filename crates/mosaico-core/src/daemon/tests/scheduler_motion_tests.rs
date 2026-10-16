//! The frame step a motion takes when it is due to end, whether or not
//! a display frame comes.

use crate::tiler::QUIET;

use super::*;

#[test]
fn the_wait_ends_with_a_frame_step_when_a_motion_is_due_to_end() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();

    // Act
    let wait = scheduler.begin_wait(None, Some(t0 + ms(40)), t0);
    let steps = scheduler.step(timed_out(), t0 + ms(40));

    // Assert -- although no frame came.
    assert_eq!(wait, Some(ms(40)));
    assert_eq!(steps, vec![Step::Frame]);
}

#[test]
fn the_end_of_a_motion_shortens_the_wait_but_not_the_quiet_window() {
    // Arrange -- a focus change is held at t0, and a motion ends 10 ms
    // later.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);

    // Act
    let wait = scheduler.begin_wait(held, Some(t0 + ms(10)), t0);
    let ended = scheduler.step(timed_out(), t0 + ms(10));
    let rest = scheduler.begin_wait(held, None, t0 + ms(11));
    let flushed = scheduler.step(timed_out(), t0 + QUIET);

    // Assert
    assert_eq!(wait, Some(ms(10)));
    assert_eq!(ended, vec![Step::Frame]);
    assert_eq!(rest, Some(QUIET - ms(11)));
    assert_eq!(flushed, vec![Step::Flush]);
}

#[test]
fn a_motion_not_due_yet_takes_no_step() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.begin_wait(None, Some(t0 + ms(40)), t0);

    // Act -- a hotkey ends the wait first.
    let steps = scheduler.step(received(vec![task("hotkey")]), t0 + ms(10));

    // Assert
    assert_eq!(steps, vec![Step::Run("hotkey")]);
}

#[test]
fn a_motion_due_when_input_ends_the_wait_takes_its_step_after_it() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.begin_wait(None, Some(t0 + ms(40)), t0);

    // Act -- the hotkey came just as the motion was due.
    let steps = scheduler.step(received(vec![task("hotkey")]), t0 + ms(40));

    // Assert
    assert_eq!(steps, vec![Step::Run("hotkey"), Step::Frame]);
}

#[test]
fn the_end_of_a_motion_left_in_the_past_takes_one_step_and_no_more() {
    // Arrange -- the step the motion took when it was due did not end it.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let due = Some(t0 + ms(40));
    scheduler.begin_wait(None, due, t0);
    let ended = scheduler.step(timed_out(), t0 + ms(40));

    // Act
    let again = scheduler.begin_wait(None, due, t0 + ms(41));
    let next = scheduler.begin_wait(None, Some(t0 + ms(60)), t0 + ms(42));

    // Assert -- the loop waits for what else comes instead of spinning,
    // and the end of the next motion is due.
    assert_eq!(ended, vec![Step::Frame]);
    assert_eq!(again, None);
    assert_eq!(next, Some(ms(18)));
}

#[test]
fn the_end_of_a_motion_a_display_frame_stepped_past_is_not_due() {
    // Arrange -- a frame came after the motion was due, before the wait
    // that would have ended for it.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.step(received(vec![Message::Frame]), t0 + ms(45));

    // Act
    let wait = scheduler.begin_wait(None, Some(t0 + ms(40)), t0 + ms(46));

    // Assert
    assert_eq!(wait, None);
}

#[test]
fn the_last_frame_step_tells_the_loop_which_motion_ends_are_still_due() {
    // Arrange -- one motion ends at 40 ms, and is still listed when the
    // step at 40 ms has been taken, and another ends at 300 ms.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let ends = [t0 + ms(40), t0 + ms(300)];
    scheduler.begin_wait(None, Some(ends[0]), t0);
    scheduler.step(timed_out(), t0 + ms(40));

    // Act -- the loop passes the earliest end after the last step.
    let stepped = scheduler.last_frame_step();
    let due = ends
        .into_iter()
        .find(|&end| stepped.is_none_or(|s| end > s));
    let wait = scheduler.begin_wait(None, due, t0 + ms(41));
    let ended = scheduler.step(timed_out(), t0 + ms(300));

    // Assert
    assert_eq!(stepped, Some(t0 + ms(40)));
    assert_eq!(wait, Some(ms(259)));
    assert_eq!(ended, vec![Step::Frame]);
    assert_eq!(scheduler.last_frame_step(), Some(t0 + ms(300)));
}
