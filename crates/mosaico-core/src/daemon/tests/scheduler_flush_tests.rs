//! When the effects the tiling manager holds back are flushed.

use crate::tiler::QUIET;

use super::*;

#[test]
fn the_quiet_window_starts_once_the_loop_is_waiting_again() {
    // Arrange -- a hotkey at t0 makes the tiler hold a focus change at
    // t0 + 1 ms, and running it takes until t0 + 30 ms.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.step(received(vec![task("hotkey")]), t0);
    let held = Some(t0 + ms(1));

    // Act
    let wait = scheduler.begin_wait(held, None, t0 + ms(30));
    let steps = scheduler.step(timed_out(), t0 + ms(30) + QUIET);

    // Assert -- running the hotkey did not count as quiet.
    assert_eq!(wait, Some(QUIET));
    assert_eq!(steps, vec![Step::Flush]);
}

#[test]
fn a_message_ends_the_wait_without_a_flush() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);
    scheduler.begin_wait(held, None, t0);

    // Act -- the message arrives 20 ms into the quiet window.
    let steps = scheduler.step(received(vec![event(focused(2))]), t0 + ms(20));
    let wait = scheduler.begin_wait(held, None, t0 + ms(21));

    // Assert -- the next wait starts the window over.
    assert_eq!(steps, vec![Step::Events(vec![focused(2)])]);
    assert_eq!(wait, Some(QUIET));
}

#[test]
fn nothing_is_flushed_while_nothing_is_held() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.step(received(vec![task("tick")]), t0);

    // Act
    let wait = scheduler.begin_wait(None, None, t0 + ms(1));
    let steps = scheduler.step(timed_out(), t0 + QUIET * 4);

    // Assert
    assert_eq!(wait, None);
    assert!(steps.is_empty(), "{steps:?}");
}

#[test]
fn a_hold_made_at_startup_waits_until_the_loop_has_waited_quietly() {
    // Arrange -- the first focus change is held as tiling starts, and
    // the rest of startup takes 30 ms before the loop first waits.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);

    // Act
    let wait = scheduler.begin_wait(held, None, t0 + ms(30));
    let steps = scheduler.step(timed_out(), t0 + ms(30) + QUIET);

    // Assert
    assert_eq!(wait, Some(QUIET));
    assert_eq!(steps, vec![Step::Flush]);
}

#[test]
fn a_hold_made_by_a_rebuild_waits_until_the_loop_has_waited_quietly() {
    // Arrange -- the rebuild holds a focus change 5 ms in and takes
    // 30 ms in all.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.step(display_changed(), t0);
    scheduler.begin_wait(None, None, t0 + SETTLE);
    let rebuild = scheduler.step(timed_out(), t0 + SETTLE);
    let done = t0 + SETTLE + ms(30);
    let held = Some(t0 + SETTLE + ms(5));

    // Act
    let wait = scheduler.begin_wait(held, None, done);
    let steps = scheduler.step(timed_out(), done + QUIET);

    // Assert
    assert_eq!(rebuild, vec![Step::RebuildMonitors]);
    assert_eq!(wait, Some(QUIET));
    assert_eq!(steps, vec![Step::Flush]);
}
