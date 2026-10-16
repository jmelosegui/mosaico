//! The wait the core [`FramePacer`] paces the frames of transitions with
//! on Win32.
//!
//! `DwmFlush` returns once DWM has composed the next frame, which follows
//! the display's refresh. It fails where nothing is composed, and returns
//! at once while the display sleeps or the session is locked, which the
//! pacer makes up on a high-resolution waitable timer.
//! `DwmGetCompositionTimingInfo` says how often DWM composes, which is
//! how the pacer tells a frame from a wait that ended too soon.
//!
//! [`FramePacer`]: mosaico_core::daemon::FramePacer

use std::thread;
use std::time::{Duration, Instant};

use mosaico_core::daemon::FrameWait;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows::Win32::Graphics::Dwm::{DWM_TIMING_INFO, DwmFlush, DwmGetCompositionTimingInfo};
use windows::Win32::System::Threading::{
    CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, CreateWaitableTimerExW, INFINITE, SetWaitableTimer,
    TIMER_ALL_ACCESS, WaitForSingleObject,
};
use windows::core::PCWSTR;

/// The [`FrameWait`] of the Win32 desktop: `DwmFlush`, and a
/// high-resolution waitable timer.
pub struct DisplayWait {
    /// `None` before Windows 10 1803, which has no high-resolution
    /// timers: then the thread sleeps instead.
    timer: Option<Timer>,
}

impl DisplayWait {
    /// Makes the wait, on the pacer's thread.
    pub fn new() -> Self {
        Self {
            timer: Timer::new(),
        }
    }
}

impl Default for DisplayWait {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameWait for DisplayWait {
    fn display(&mut self) -> bool {
        // SAFETY: DwmFlush takes no arguments, and returns once DWM has
        // composed its next frame, or fails.
        unsafe { DwmFlush() }.is_ok()
    }

    fn until(&mut self, at: Instant) {
        let left = at.saturating_duration_since(Instant::now());
        if !self.timer.as_ref().is_some_and(|timer| timer.wait(left)) {
            thread::sleep(left);
        }
    }

    fn now(&self) -> Instant {
        Instant::now()
    }

    /// How long DWM takes to compose a frame, from the refresh rate it
    /// reports for the desktop.
    fn refresh(&self) -> Option<Duration> {
        let mut info = DWM_TIMING_INFO {
            cbSize: size_of::<DWM_TIMING_INFO>() as u32,
            ..Default::default()
        };
        // SAFETY: fills a struct on the stack whose size it is given. The
        // window must be null, for the whole desktop.
        unsafe { DwmGetCompositionTimingInfo(HWND::default(), &mut info) }.ok()?;
        let rate = info.rateRefresh;
        let (frames, seconds) = (rate.uiNumerator, rate.uiDenominator);
        (frames > 0 && seconds > 0)
            .then(|| Duration::from_secs_f64(f64::from(seconds) / f64::from(frames)))
    }
}

/// A high-resolution waitable timer, which wakes within a fraction of a
/// millisecond of its time, where a plain sleep can be rounded up to the
/// system timer's tick of about 15.6 ms.
struct Timer(HANDLE);

impl Timer {
    fn new() -> Option<Self> {
        // SAFETY: creates an unnamed timer with the default security; the
        // handle is closed on drop.
        let handle = unsafe {
            CreateWaitableTimerExW(
                None,
                PCWSTR::null(),
                CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                TIMER_ALL_ACCESS.0,
            )
        };
        handle.ok().map(Self)
    }

    /// Waits for `left`. False when the timer could not be set.
    fn wait(&self, left: Duration) -> bool {
        // Negative for a time relative to now, in units of 100 ns.
        let due = -i64::try_from(left.as_nanos() / 100).unwrap_or(i64::MAX);
        // SAFETY: sets the timer this owns, without a completion routine,
        // from a due time that outlives the call.
        if unsafe { SetWaitableTimer(self.0, &due, 0, None, None, false) }.is_err() {
            return false;
        }
        // SAFETY: waits on the timer this owns, which was just set.
        unsafe {
            WaitForSingleObject(self.0, INFINITE);
        }
        true
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        // SAFETY: closes the handle this owns, once.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

#[cfg(test)]
#[path = "frame_clock_dwm_tests.rs"]
mod tests;
