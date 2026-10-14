//! Focus-follows-mouse: noticing where the pointer comes to rest.

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::{Result, WindowEvent, WindowId};

use super::rest::HoverTracker;

/// How often the pointer is looked at while focus-follows-mouse is on.
pub const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// How many polls in a row must find the pointer on the same window
/// before that window is reported.
pub const REST_POLLS: u8 = 3;

/// Tells which window is under the mouse pointer.
///
/// # Contract for platform implementations
///
/// - `window_under` answers with the top-level window under the pointer,
///   the one the user would say they are pointing at, whether it is
///   managed or not. Never a control or other element inside a window.
/// - It answers `Ok(None)` when the pointer is over no window, and fails
///   when the pointer cannot be read right now. Focus-follows-mouse skips
///   such polls.
/// - It is a query: it must not move the pointer, focus a window or
///   change which window is on top.
/// - It is called every [`POLL_INTERVAL`] from a thread of its own while
///   focus-follows-mouse is on, so it must be cheap.
/// - A platform that cannot tell which window is under the pointer fails
///   with [`Error::Unsupported`](crate::Error::Unsupported), and
///   focus-follows-mouse then never focuses anything.
pub trait MousePointer: Send + Sync {
    /// The window under the pointer.
    fn window_under(&self) -> Result<Option<WindowId>>;
}

/// Focus-follows-mouse, reporting each window the pointer comes to rest
/// on as [`WindowEvent::MouseHover`].
///
/// While it is on, a thread of its own looks at the pointer every
/// [`POLL_INTERVAL`] and reports a window once the pointer has stayed on
/// it for [`REST_POLLS`] polls in a row. While it is off, no thread runs.
pub struct FocusFollowsMouse {
    pointer: Arc<dyn MousePointer>,
    events: Sender<WindowEvent>,
    interval: Duration,
    poller: Option<Poller>,
}

impl FocusFollowsMouse {
    /// Focus-follows-mouse over `pointer`, reporting to `events`. It
    /// starts off.
    pub fn new(pointer: impl MousePointer + 'static, events: Sender<WindowEvent>) -> Self {
        Self::polling_every(POLL_INTERVAL, pointer, events)
    }

    fn polling_every(
        interval: Duration,
        pointer: impl MousePointer + 'static,
        events: Sender<WindowEvent>,
    ) -> Self {
        Self {
            pointer: Arc::new(pointer),
            events,
            interval,
            poller: None,
        }
    }

    /// Whether it is on.
    pub fn is_enabled(&self) -> bool {
        self.poller.is_some()
    }

    /// Turns it on or off.
    ///
    /// Turning it on starts the polling thread, unless it is running
    /// already. Turning it off stops the thread and waits for it, which
    /// forgets where the pointer was resting. Fails only when the thread
    /// cannot be started.
    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        if !enabled {
            if let Some(poller) = self.poller.take() {
                poller.stop();
            }
        } else if self.poller.is_none() {
            let pointer = Arc::clone(&self.pointer);
            let events = self.events.clone();
            self.poller = Some(Poller::start(pointer, events, self.interval)?);
        }
        Ok(())
    }
}

/// Dropping it turns it off and waits for the thread.
impl Drop for FocusFollowsMouse {
    fn drop(&mut self) {
        if let Some(poller) = self.poller.take() {
            poller.stop();
        }
    }
}

/// The polling thread, while focus-follows-mouse is on.
struct Poller {
    /// Closing it ends the thread's wait between polls.
    stop: Sender<()>,
    thread: JoinHandle<()>,
}

impl Poller {
    fn start(
        pointer: Arc<dyn MousePointer>,
        events: Sender<WindowEvent>,
        interval: Duration,
    ) -> Result<Self> {
        let (stop, stopped) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("focus-follows-mouse".to_string())
            .spawn(move || poll(pointer.as_ref(), &events, &stopped, interval))?;
        Ok(Self { stop, thread })
    }

    fn stop(self) {
        drop(self.stop);
        let _ = self.thread.join();
    }
}

/// Looks at the pointer every `interval` until `stopped` closes.
fn poll(
    pointer: &dyn MousePointer,
    events: &Sender<WindowEvent>,
    stopped: &Receiver<()>,
    interval: Duration,
) {
    let mut rest = HoverTracker::default();
    while let Err(RecvTimeoutError::Timeout) = stopped.recv_timeout(interval) {
        // A pointer that cannot be read is skipped like one over nothing.
        let under = pointer.window_under().unwrap_or(None);
        if let Some(window) = rest.observe(under) {
            let _ = events.send(WindowEvent::MouseHover { window });
        }
    }
}

#[cfg(test)]
#[path = "tests/hover_tests.rs"]
mod tests;
