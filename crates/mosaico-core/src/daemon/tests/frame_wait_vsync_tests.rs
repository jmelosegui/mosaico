//! Waiting for the next frame on a simulated display, whose refreshes
//! come on time from its start, for a pacer that is woken late once.

use super::*;

/// A display that refreshes every `period` from `start`. A wait on it
/// ends at its next refresh, and once `late` after it when that is set,
/// as when the pacer's thread is woken late.
struct Vsync {
    start: Instant,
    now: Instant,
    period: Duration,
    late: Option<Duration>,
    timer: Vec<Instant>,
}

impl Vsync {
    fn new(period: Duration) -> Self {
        let start = Instant::now();
        Self {
            start,
            now: start,
            period,
            late: None,
            timer: Vec::new(),
        }
    }

    /// When refresh `n` comes.
    fn refresh_at(&self, n: u32) -> Instant {
        self.start + self.period * n
    }
}

impl FrameWait for Vsync {
    fn display(&mut self) -> bool {
        let since = self.now.duration_since(self.start);
        let next = since.as_nanos() / self.period.as_nanos() + 1;
        let next = u32::try_from(next).expect("a short test");
        self.now = self.refresh_at(next) + self.late.take().unwrap_or_default();
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
        Some(self.period)
    }
}

#[test]
fn a_late_wake_costs_one_frame_and_the_next_ones_keep_to_the_display() {
    // Arrange -- at 144 Hz, the pacer is woken for the first refresh
    // 4 ms late, more than half a refresh.
    let mut display = Vsync::new(HZ_144);
    let late = Duration::from_millis(4);
    display.late = Some(late);

    // Act
    let mut last = None;
    let mut frames = Vec::new();
    for _ in 0..4 {
        let at = next_frame(&mut display, last);
        frames.push(at);
        last = Some(at);
    }

    // Assert -- the second refresh came too soon after the late wake,
    // and the next was waited for on the display, not the timer.
    let on_time = [3, 4, 5].map(|n| display.refresh_at(n));
    assert_eq!(frames[0], display.refresh_at(1) + late);
    assert_eq!(frames[1..], on_time);
    assert!(display.timer.is_empty(), "{:?}", display.timer);
}
