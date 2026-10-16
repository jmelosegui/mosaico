//! Waiting for the next frame, on a fake clock.

use super::*;

#[path = "frame_wait_vsync_tests.rs"]
mod vsync_tests;

/// A wait on a fake clock. The display holds each wait back for `paced`,
/// or cannot be waited for when it is `None`, and says it refreshes every
/// `refresh`. The timer moves the clock on to its time.
struct FakeWait {
    now: Instant,
    paced: Option<Duration>,
    refresh: Option<Duration>,
    timer: Vec<Instant>,
}

impl FakeWait {
    fn new(paced: Option<Duration>, refresh: Option<Duration>) -> Self {
        Self {
            now: Instant::now(),
            paced,
            refresh,
            timer: Vec::new(),
        }
    }
}

impl FrameWait for FakeWait {
    fn display(&mut self) -> bool {
        let Some(paced) = self.paced else {
            return false;
        };
        self.now += paced;
        true
    }

    fn until(&mut self, at: Instant) {
        self.timer.push(at);
        self.now = self.now.max(at);
    }

    fn now(&self) -> Instant {
        self.now
    }

    fn refresh(&self) -> Option<Duration> {
        self.refresh
    }
}

fn us(n: u64) -> Duration {
    Duration::from_micros(n)
}

/// The display of a 144 Hz monitor.
const HZ_144: Duration = Duration::from_micros(6_944);

#[test]
fn a_display_that_holds_the_wait_back_sets_the_pace() {
    // Arrange
    let mut wait = FakeWait::new(Some(HZ_144), Some(HZ_144));
    let last = wait.now;

    // Act
    let at = next_frame(&mut wait, Some(last));

    // Assert
    assert_eq!(at, last + HZ_144);
    assert!(wait.timer.is_empty(), "{:?}", wait.timer);
}

#[test]
fn a_frame_a_little_less_than_a_refresh_after_the_last_follows_the_display() {
    // Arrange -- the last frame came late.
    let mut wait = FakeWait::new(Some(us(4_000)), Some(HZ_144));
    let last = wait.now;

    // Act
    let at = next_frame(&mut wait, Some(last));

    // Assert
    assert_eq!(at, last + us(4_000));
    assert!(wait.timer.is_empty(), "{:?}", wait.timer);
}

#[test]
fn without_a_display_to_wait_for_frames_come_about_60_times_a_second() {
    // Arrange
    let mut wait = FakeWait::new(None, None);
    let last = wait.now;

    // Act
    let at = next_frame(&mut wait, Some(last));

    // Assert
    assert_eq!(at, last + FALLBACK_FRAME);
    assert_eq!(wait.timer, vec![last + FALLBACK_FRAME]);
}

#[test]
fn a_display_that_lets_two_waits_in_a_row_end_at_once_is_made_up_at_its_refresh() {
    // Arrange -- the display sleeps, or the session is locked, and each
    // wait ends well within half a refresh.
    let mut wait = FakeWait::new(Some(us(2_000)), Some(HZ_144));
    let last = wait.now;

    // Act
    let at = next_frame(&mut wait, Some(last));

    // Assert -- a refresh after the last frame, on the timer.
    assert_eq!(at, last + HZ_144);
    assert_eq!(wait.timer, vec![last + HZ_144]);
}

#[test]
fn a_display_whose_refresh_is_not_known_is_too_soon_within_a_millisecond() {
    // Arrange
    let mut late = FakeWait::new(Some(us(1_500)), None);
    let mut soon = FakeWait::new(Some(us(500)), None);
    let (from_late, from_soon) = (late.now, soon.now);

    // Act
    let late_at = next_frame(&mut late, Some(from_late));
    let soon_at = next_frame(&mut soon, Some(from_soon));

    // Assert
    assert_eq!(late_at, from_late + us(1_500));
    assert_eq!(soon_at, from_soon + FALLBACK_FRAME);
}

#[test]
fn the_first_frame_after_the_pacer_slept_takes_what_the_display_gives() {
    // Arrange -- the display is about to refresh.
    let mut wait = FakeWait::new(Some(us(500)), Some(HZ_144));
    let start = wait.now;

    // Act
    let at = next_frame(&mut wait, None);

    // Assert
    assert_eq!(at, start + us(500));
    assert!(wait.timer.is_empty(), "{:?}", wait.timer);
}
