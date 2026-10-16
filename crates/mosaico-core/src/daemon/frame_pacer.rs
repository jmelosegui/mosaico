//! The display frames that pace transitions, from a thread of their own.
//!
//! [`FramePacer`] is the [`FrameClock`] of any platform that can wait for
//! its display's frames through a [`FrameWait`]. One thread, started with
//! the pacer and ended with it, is parked while no frames are wanted.
//! While they are, it waits for each frame and sends it with a
//! [`FrameToken`], and no other is sent until the loop has dropped that
//! token, so at most one frame waits in the loop's queue. When a frame
//! cannot be sent, as once the loop is gone, the thread parks until
//! frames are asked for again.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::Result;

use super::FrameClock;
use super::frame_wait::{FrameWait, next_frame};

/// How long stopping the pacer waits for its thread, which is at most a
/// frame into a wait. A thread that takes longer, waiting on a display
/// that does not come back, is left to end by itself.
const STOP_WAIT: Duration = Duration::from_millis(100);

/// What the pacer and its thread share. Neither side ever blocks the
/// other.
#[derive(Debug, Default)]
struct Shared {
    /// Whether frames are wanted.
    wanted: AtomicBool,
    /// Whether a frame was sent that the loop has not taken yet.
    sent: AtomicBool,
    /// Whether the pacer is stopping.
    stopping: AtomicBool,
}

/// The frame a [`FramePacer`] sent, until the loop takes it.
///
/// No other frame is sent while it exists. The loop drops it when it
/// takes the frame out of its queue, which lets the next one go.
#[derive(Debug)]
pub struct FrameToken(Arc<Shared>);

impl Drop for FrameToken {
    fn drop(&mut self) {
        self.0.sent.store(false, Ordering::Release);
    }
}

/// A [`FrameClock`] on a thread that sends a frame on every refresh of
/// the display while frames are wanted.
#[derive(Debug)]
pub struct FramePacer {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
    /// Disconnects once the thread has ended.
    ended: mpsc::Receiver<()>,
}

impl FramePacer {
    /// Starts the pacer's thread, which makes its wait with `make_wait`,
    /// sleeps until frames are wanted, and then hands each frame to
    /// `send`, which says whether the frame reached the loop. Fails when
    /// the thread cannot be started.
    pub fn start<W: FrameWait>(
        make_wait: impl FnOnce() -> W + Send + 'static,
        send: impl Fn(FrameToken) -> bool + Send + 'static,
    ) -> Result<Self> {
        let shared = Arc::new(Shared::default());
        let (ended_tx, ended) = mpsc::channel::<()>();
        let theirs = Arc::clone(&shared);
        let thread = thread::Builder::new()
            .name("frames".to_string())
            .spawn(move || {
                let _ended = ended_tx;
                run(&theirs, &mut make_wait(), &send);
            })?;
        Ok(Self {
            shared,
            thread: Some(thread),
            ended,
        })
    }
}

impl FrameClock for FramePacer {
    fn want_frames(&self, on: bool) {
        self.shared.wanted.store(on, Ordering::Release);
        if on && let Some(thread) = &self.thread {
            thread.thread().unpark();
        }
    }
}

/// Stops the thread within a frame, without waiting on a display that
/// does not come back.
impl Drop for FramePacer {
    fn drop(&mut self) {
        self.shared.stopping.store(true, Ordering::Release);
        let Some(thread) = self.thread.take() else {
            return;
        };
        thread.thread().unpark();
        if self.ended.recv_timeout(STOP_WAIT) == Err(RecvTimeoutError::Disconnected) {
            let _ = thread.join();
        }
    }
}

/// The pacer's thread: sleeps while no frames are wanted, and otherwise
/// sends one per frame, unless the last one has not been taken. A frame
/// that could not be sent parks it until it is woken.
fn run(shared: &Arc<Shared>, wait: &mut impl FrameWait, send: &impl Fn(FrameToken) -> bool) {
    let mut last = None;
    while !shared.stopping.load(Ordering::Acquire) {
        if !shared.wanted.load(Ordering::Acquire) {
            thread::park();
            last = None;
            continue;
        }
        last = Some(next_frame(wait, last));
        if shared.wanted.load(Ordering::Acquire)
            && !shared.sent.swap(true, Ordering::AcqRel)
            && !send(FrameToken(Arc::clone(shared)))
        {
            thread::park();
            last = None;
        }
    }
}

#[cfg(test)]
#[path = "tests/frame_pacer_tests.rs"]
mod tests;
