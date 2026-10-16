//! What counts as input for the quiet window: every message but a
//! window's report that its position changed.

use crate::tiler::{MAX_HOLD, QUIET};

use super::*;

#[test]
fn a_position_report_does_not_start_the_quiet_window_over() {
    // Arrange -- a focus change is held at t0.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);
    scheduler.begin_wait(held, None, t0);

    // Act -- a window reports its position 10 ms into the quiet window.
    let steps = scheduler.step(received(vec![event(moved(1))]), t0 + ms(10));
    let wait = scheduler.begin_wait(held, None, t0 + ms(11));

    // Assert -- the report still reaches the tiler, which watches for
    // the focused window being maximized, and the wait goes on.
    assert_eq!(steps, vec![Step::Events(vec![moved(1)])]);
    assert_eq!(wait, Some(QUIET - ms(11)));
}

#[test]
fn held_effects_are_flushed_after_the_quiet_window_while_a_window_reports_its_position() {
    // Arrange -- a move is held at t0 while a window reports its position
    // every 7 ms, as one does while it animates, or while the tiler's own
    // placements land. Each report used to end the quiet window, so the
    // move waited for the cap.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);
    let mut now = t0;

    // Act -- waited for the way the daemon loop waits: whichever comes
    // first, the end of the wait or the next report.
    let mut flushed_at = None;
    while flushed_at.is_none() && now < t0 + MAX_HOLD {
        let next_report = now + ms(7);
        let wake = match scheduler.begin_wait(held, None, now) {
            Some(left) if now + left < next_report => {
                now += left;
                timed_out()
            }
            _ => {
                now = next_report;
                received(vec![event(moved(1))])
            }
        };
        if scheduler.step(wake, now).contains(&Step::Flush) {
            flushed_at = Some(now);
        }
    }

    // Assert
    assert_eq!(flushed_at, Some(t0 + QUIET));
}

#[test]
fn a_position_report_finds_what_is_due_by_the_time_it_arrives() {
    // Arrange -- the loop gets to the report after the quiet window.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.begin_wait(Some(t0), None, t0);

    // Act
    let steps = scheduler.step(received(vec![event(moved(1))]), t0 + ms(30));

    // Assert -- as a timeout would have, and before the report.
    assert_eq!(steps, vec![Step::Flush, Step::Events(vec![moved(1)])]);
}

#[test]
fn position_reports_that_come_with_input_are_input() {
    // Arrange
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);
    scheduler.begin_wait(held, None, t0);

    // Act
    let batch = received(vec![event(moved(1)), event(focused(2))]);
    let steps = scheduler.step(batch, t0 + ms(10));
    let wait = scheduler.begin_wait(held, None, t0 + ms(11));

    // Assert -- the focus change starts the quiet window over.
    assert_eq!(steps, vec![Step::Events(vec![moved(1), focused(2)])]);
    assert_eq!(wait, Some(QUIET));
}

#[test]
fn a_hold_made_after_position_reports_flushed_the_last_gets_its_own_quiet_window() {
    // Arrange -- reports that come after the quiet window flush the hold.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    scheduler.begin_wait(Some(t0), None, t0);
    let flush = scheduler.step(received(vec![event(moved(1))]), t0 + ms(30));

    // Act -- a new focus change is held as the flush runs.
    let new_hold = Some(t0 + ms(31));
    let wait = scheduler.begin_wait(new_hold, None, t0 + ms(32));

    // Assert
    assert_eq!(flush.first(), Some(&Step::Flush));
    assert_eq!(wait, Some(QUIET));
}

#[test]
fn the_time_a_batch_of_position_reports_takes_counts_as_quiet() {
    // Arrange -- a focus change is held at t0, and the reports of a
    // window 10 ms later take the loop 20 ms to handle.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);
    scheduler.begin_wait(held, None, t0);
    scheduler.step(received(vec![event(moved(1))]), t0 + ms(10));

    // Act
    let wait = scheduler.begin_wait(held, None, t0 + ms(30));

    // Assert -- the quiet window passed while they were handled.
    assert_eq!(wait, Some(Duration::ZERO));
}
