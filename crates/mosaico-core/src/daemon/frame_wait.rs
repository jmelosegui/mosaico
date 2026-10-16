//! What a [`FramePacer`](super::FramePacer) waits on, and how it finds
//! the next frame from it.

use std::time::{Duration, Instant};

/// A frame at about 60 frames a second, when the platform cannot say how
/// often the display refreshes.
const FALLBACK_FRAME: Duration = Duration::from_micros(16_667);

/// How soon after the last frame a wait on the display may end and
/// still be one of its frames, when the platform cannot say how often
/// the display refreshes: no display refreshes a thousand times a second.
const TOO_SOON_FALLBACK: Duration = Duration::from_millis(1);

/// The display's frames, and a timer, as a
/// [`FramePacer`](super::FramePacer) waits on them on its own thread.
///
/// # Contract for platform implementations
///
/// - [`display`](Self::display) returns once the display has shown its
///   next frame, or false at once when it cannot be waited for. It may
///   return too soon, as while the display sleeps or the session is
///   locked: the pacer makes that up on the timer.
/// - [`until`](Self::until) returns at `at`, within a fraction of a
///   millisecond where the platform can, and at once when `at` has
///   passed.
/// - [`now`](Self::now) reads the clock both are on.
/// - [`refresh`](Self::refresh) is how long the display takes to
///   refresh, or `None` when the platform cannot say.
pub trait FrameWait {
    /// Waits for the display's next frame. False when it cannot be waited
    /// for.
    fn display(&mut self) -> bool;

    /// Waits until `at`.
    fn until(&mut self, at: Instant);

    /// The time now.
    fn now(&self) -> Instant;

    /// How long the display takes to refresh, if the platform knows.
    fn refresh(&self) -> Option<Duration>;
}

/// How soon after the last frame a wait on the display may end and still
/// be one of its frames: half a refresh, since a frame comes about one
/// refresh after the last, and a little sooner when the last came late.
pub(super) fn too_soon(refresh: Option<Duration>) -> Duration {
    refresh.map_or(TOO_SOON_FALLBACK, |refresh| refresh / 2)
}

/// Waits for the next frame after the one at `last`, if there was one,
/// and returns when it came.
///
/// That is the display's next frame. A wait that ends too soon after the
/// last frame, as one does after a frame the pacer woke for late, is
/// waited once more. When that ends too soon as well, as while the
/// display sleeps, or the display cannot be waited for, the rest of a
/// refresh is waited out on the timer, or of a frame at about 60 a second
/// when the platform cannot say how often the display refreshes. The
/// first frame after the pacer slept takes what the display gives.
pub(super) fn next_frame(wait: &mut impl FrameWait, last: Option<Instant>) -> Instant {
    let refresh = wait.refresh().filter(|refresh| !refresh.is_zero());
    let soon = too_soon(refresh);
    let mut from = last;
    for _ in 0..2 {
        if !wait.display() {
            break;
        }
        let now = wait.now();
        if from.is_none_or(|from| now.duration_since(from) >= soon) {
            return now;
        }
        from = Some(now);
    }
    let period = refresh.unwrap_or(FALLBACK_FRAME);
    let now = wait.now();
    let at = match last {
        Some(last) => (last + period).max(now),
        None => now + period,
    };
    wait.until(at);
    at
}

#[cfg(test)]
#[path = "tests/frame_wait_tests.rs"]
mod tests;
