//! When a display change is rebuilt for.

use crate::tiler::QUIET;

use super::*;

#[test]
fn a_display_change_waits_for_the_configuration_to_settle() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let batch = received(vec![
        event(WindowEvent::DisplayChanged),
        event(WindowEvent::WorkAreaChanged),
        event(focused(1)),
    ]);

    // Act
    let arrived = scheduler.step(batch, t0);
    let early = scheduler.step(timed_out(), t0 + SETTLE / 2);
    let settled = scheduler.step(timed_out(), t0 + SETTLE);
    let later = scheduler.step(timed_out(), t0 + SETTLE * 2);

    // Assert -- the loop is told at once, to stop what moves to the old
    // layout, and the rest of the batch does not wait.
    let events = vec![WindowEvent::WorkAreaChanged, focused(1)];
    assert_eq!(arrived, vec![Step::DisplayChanging, Step::Events(events)]);
    assert!(early.is_empty(), "rebuilt before settling: {early:?}");
    assert_eq!(settled, vec![Step::RebuildMonitors]);
    assert!(later.is_empty(), "rebuilt twice: {later:?}");
}

#[test]
fn a_burst_of_display_changes_is_rebuilt_for_once_after_the_last() {
    // Arrange -- a KVM switch reports several changes in a row.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.step(display_changed(), t0);
    scheduler.step(display_changed(), t0 + ms(500));

    // Act
    let after_first = scheduler.step(timed_out(), t0 + SETTLE);
    let after_last = scheduler.step(timed_out(), t0 + ms(500) + SETTLE);

    // Assert
    assert!(after_first.is_empty(), "rebuilt mid-burst: {after_first:?}");
    assert_eq!(after_last, vec![Step::RebuildMonitors]);
}

#[test]
fn the_wait_ends_when_a_display_change_settles() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.step(display_changed(), t0);

    // Act
    let wait = scheduler.begin_wait(None, None, None, t0 + ms(250));

    // Assert
    assert_eq!(wait, Some(SETTLE - ms(250)));
}

#[test]
fn a_display_change_settled_before_the_wait_is_rebuilt_for_first() {
    // Arrange -- the change settled while the loop was busy.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.step(display_changed(), t0);
    let held = Some(t0 + SETTLE - ms(5));

    // Act
    let wait = scheduler.begin_wait(held, None, None, t0 + SETTLE + ms(5));
    let batch = received(vec![task("hotkey"), event(focused(1))]);
    let steps = scheduler.step(batch, t0 + SETTLE + ms(5));

    // Assert -- not even held effects delay it.
    assert_eq!(wait, Some(Duration::ZERO));
    assert_eq!(
        steps,
        vec![
            Step::RebuildMonitors,
            Step::Run("hotkey"),
            Step::Events(vec![focused(1)]),
        ]
    );
}

#[test]
fn while_effects_are_held_the_wait_is_for_them_alone() {
    // Arrange -- the change settles 10 ms into the quiet window.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.step(display_changed(), t0);
    let start = t0 + SETTLE - ms(10);

    // Act
    let wait = scheduler.begin_wait(Some(start), None, None, start);

    // Assert
    assert_eq!(wait, Some(QUIET));
}

#[test]
fn a_flush_due_with_a_rebuild_comes_first() {
    // Arrange -- a rebuild can hide the window a held focus change is
    // for, so the focus change lands first, as it always has.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.step(display_changed(), t0);
    let start = t0 + SETTLE - ms(10);
    scheduler.begin_wait(Some(start), None, None, start);

    // Act
    let steps = scheduler.step(timed_out(), start + QUIET);

    // Assert
    assert_eq!(steps, vec![Step::Flush, Step::RebuildMonitors]);
}

#[test]
fn a_display_change_that_settles_during_a_wait_waits_for_the_batch_that_ends_it() {
    // Arrange -- effects are held, and a message arrives after the
    // change has settled but before the flush is due.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.step(display_changed(), t0);
    let start = t0 + SETTLE - ms(10);
    let held = Some(start);
    scheduler.begin_wait(held, None, None, start);

    // Act
    let batch = scheduler.step(received(vec![task("hotkey")]), t0 + SETTLE + ms(5));
    let wait = scheduler.begin_wait(held, None, None, t0 + SETTLE + ms(6));
    let next = scheduler.step(timed_out(), t0 + SETTLE + ms(6));

    // Assert -- the batch runs first and the rebuild right after it, as
    // the loop always did.
    assert_eq!(batch, vec![Step::Run("hotkey")]);
    assert_eq!(wait, Some(Duration::ZERO));
    assert_eq!(next, vec![Step::RebuildMonitors]);
}
