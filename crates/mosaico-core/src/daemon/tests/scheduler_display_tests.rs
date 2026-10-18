//! Where a display change comes among what a wake runs: first, so what
//! moves stops before anything else lays windows out for the old layout.

use crate::tiler::MAX_HOLD;

use super::*;

#[test]
fn a_display_change_in_the_wake_that_flushes_is_noted_before_the_flush() {
    // Arrange -- messages have ended every wait early since the loop
    // first waited with a swap held, so the cap has passed.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);
    scheduler.begin_wait(held, None, None, t0);
    scheduler.step(received(vec![event(titled(1))]), t0 + ms(10));

    // Act -- the display changes as the loop gets back to waiting.
    let wait = scheduler.begin_wait(held, None, None, t0 + MAX_HOLD);
    let steps = scheduler.step(display_changed(), t0 + MAX_HOLD);

    // Assert -- what moves stops first, so the swap the flush releases is
    // not stopped before its first frame.
    assert_eq!(wait, Some(Duration::ZERO));
    assert_eq!(steps, vec![Step::DisplayChanging, Step::Flush]);
}

#[test]
fn a_display_change_in_the_wake_that_rebuilds_is_noted_again_after_the_rebuild() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.step(display_changed(), t0);

    // Act -- another change comes as the first has settled.
    let wait = scheduler.begin_wait(None, None, None, t0 + SETTLE);
    let steps = scheduler.step(display_changed(), t0 + SETTLE);
    let later = scheduler.step(timed_out(), t0 + SETTLE * 2);

    // Assert -- the rebuild ends what the first change stopped, and the
    // second is still settling until its own rebuild.
    assert_eq!(wait, Some(Duration::ZERO));
    assert_eq!(
        steps,
        vec![
            Step::DisplayChanging,
            Step::RebuildMonitors,
            Step::DisplayChanging
        ]
    );
    assert_eq!(later, vec![Step::RebuildMonitors]);
}
