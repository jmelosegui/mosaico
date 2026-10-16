//! The cap on how long the effects the tiler holds back can wait.

use crate::tiler::{MAX_HOLD, QUIET};

use super::*;

/// Waits with the effects held since `held`, from `from` until `until`,
/// with every wait ended early by a message 10 ms in.
fn interrupted_waits(
    scheduler: &mut Scheduler,
    held: Option<Instant>,
    from: Instant,
    until: Instant,
) {
    let mut now = from;
    while now < until {
        scheduler.begin_wait(held, now);
        now += ms(10);
        scheduler.step(received(vec![event(moved(1))]), now);
    }
}

#[test]
fn once_the_max_hold_has_passed_the_flush_runs_before_the_next_batch() {
    // Arrange -- #74: messages have ended every wait early since the loop
    // first waited with a focus change held, and more are queued when it
    // gets back to waiting as the cap passes.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);
    interrupted_waits(&mut scheduler, held, t0, t0 + MAX_HOLD);

    // Act
    let wait = scheduler.begin_wait(held, t0 + MAX_HOLD);
    let batch = received(vec![task("stop"), event(moved(1))]);
    let steps = scheduler.step(batch, t0 + MAX_HOLD);

    // Assert -- the flush comes first, so it never runs after a stop
    // that arrived with the batch.
    assert_eq!(wait, Some(Duration::ZERO));
    assert_eq!(
        steps,
        vec![Step::Flush, Step::Run("stop"), Step::Events(vec![moved(1)])]
    );
}

#[test]
fn the_wait_never_outlasts_the_max_hold() {
    // Arrange -- messages have ended every wait early for 90 ms.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);
    interrupted_waits(&mut scheduler, held, t0, t0 + ms(90));

    // Act
    let wait = scheduler.begin_wait(held, t0 + ms(90));
    let steps = scheduler.step(timed_out(), t0 + MAX_HOLD);

    // Assert -- shorter than the quiet window would have been.
    assert_eq!(wait, Some(MAX_HOLD - ms(90)));
    assert_eq!(steps, vec![Step::Flush]);
}

#[test]
fn the_cap_counts_from_the_first_wait_with_the_hold() {
    // Arrange -- a batch holds a focus change at t0 and takes 60 ms, and
    // from then on messages end every wait early.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);
    interrupted_waits(&mut scheduler, held, t0 + ms(60), t0 + ms(140));

    // Act
    let wait = scheduler.begin_wait(held, t0 + ms(140));

    // Assert -- 100 ms after the first wait, not after the hold began.
    assert_eq!(wait, Some(ms(20)));
}

#[test]
fn a_batch_slower_than_the_cap_keeps_its_quiet_window() {
    // Arrange -- a workspace switch holds its focus change at t0 + 1 ms
    // and takes until t0 + 120 ms, with the next switch already queued.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.step(received(vec![task("switch to 2")]), t0);
    let held = Some(t0 + ms(1));

    // Act
    let wait = scheduler.begin_wait(held, t0 + ms(120));
    let steps = scheduler.step(received(vec![task("switch to 3")]), t0 + ms(120));

    // Assert -- the queued switch runs and replaces the held focus
    // change, as it always did.
    assert_eq!(wait, Some(QUIET));
    assert_eq!(steps, vec![Step::Run("switch to 3")]);
}

#[test]
fn a_slow_startup_keeps_its_quiet_window() {
    // Arrange -- the first focus change is held as tiling starts, and
    // the rest of startup takes 200 ms.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);

    // Act
    let wait = scheduler.begin_wait(held, t0 + ms(200));
    let steps = scheduler.step(timed_out(), t0 + ms(200) + QUIET);

    // Assert
    assert_eq!(wait, Some(QUIET));
    assert_eq!(steps, vec![Step::Flush]);
}

#[test]
fn a_hold_started_by_the_batch_that_ends_a_wait_gets_its_own_quiet_window() {
    // Arrange -- messages have ended every wait early for 90 ms. The
    // batch that arrives as the cap passes closes the window, which ends
    // that hold, and the tiler holds a focus change for the next window.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);
    interrupted_waits(&mut scheduler, held, t0, t0 + ms(90));
    scheduler.begin_wait(held, t0 + ms(90));

    // Act
    let steps = scheduler.step(received(vec![task("close")]), t0 + MAX_HOLD);
    let new_hold = Some(t0 + MAX_HOLD + ms(1));
    let wait = scheduler.begin_wait(new_hold, t0 + MAX_HOLD + ms(2));

    // Assert -- nothing flushes the new focus change at once.
    assert_eq!(steps, vec![Step::Run("close")]);
    assert_eq!(wait, Some(QUIET));
}
