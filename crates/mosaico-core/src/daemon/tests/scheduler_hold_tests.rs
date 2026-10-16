//! How long the effects the tiling manager holds back can be kept
//! waiting.

use super::*;

#[test]
fn a_steady_stream_of_events_does_not_hold_effects_back_forever() {
    // Arrange -- #74: a focus change is held at t0 while a window
    // reports its position every 10 ms, as it does while it animates.
    let t0 = Instant::now();
    let mut scheduler = Scheduler::default();
    let held = Some(t0);
    let mut now = t0;

    // Act -- a second of that stream, waited for the way the daemon loop
    // waits: whichever comes first, the end of the wait or the event.
    let mut flushed = false;
    for _ in 0..100 {
        let next_event = now + ms(10);
        let wake = match scheduler.begin_wait(held, None, now) {
            Some(left) if now + left < next_event => {
                now += left;
                timed_out()
            }
            _ => {
                now = next_event;
                received(vec![event(moved(1))])
            }
        };
        if scheduler.step(wake, now).contains(&Step::Flush) {
            flushed = true;
            break;
        }
    }

    // Assert
    assert!(
        flushed,
        "the held focus change was never applied while events kept arriving"
    );
}
