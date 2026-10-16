//! Threads that own Win32 state delivered through a message queue.

use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use mosaico_core::{Error, Result};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PM_NOREMOVE, PeekMessageW, PostThreadMessageW,
    TranslateMessage, WM_QUIT, WM_USER,
};

/// How long `stop` waits before it posts `WM_QUIT` again to a thread
/// whose queue is full.
const QUIT_RETRY: Duration = Duration::from_millis(10);

/// A thread with a message queue, for what Windows delivers to the
/// thread that set it up: window event hooks, hotkeys and windows.
///
/// The thread's body sets up what it owns, reports how that went, pumps
/// messages until the thread is stopped, and then takes down what it set
/// up before the thread ends.
pub(crate) struct MessageThread {
    thread_id: u32,
    thread: Option<JoinHandle<()>>,
}

/// How the body of a [`MessageThread`] reports that its setup is done,
/// along with anything the caller should know about it.
pub(crate) struct Ready<T> {
    thread_id: u32,
    sender: mpsc::Sender<Result<(u32, T)>>,
}

impl<T> Ready<T> {
    /// Reports the setup. On an error, `spawn` returns it, and the body
    /// must return without pumping.
    pub(crate) fn report(self, setup: Result<T>) {
        let _ = self.sender.send(setup.map(|value| (self.thread_id, value)));
    }
}

impl MessageThread {
    /// Runs `body` on a new thread called `name`, and returns once the
    /// body has reported its setup, with what the body reported. The
    /// thread's queue exists before the body runs, so nothing posted to it
    /// afterwards is lost.
    pub(crate) fn spawn<T: Send + 'static>(
        name: &str,
        body: impl FnOnce(Ready<T>) + Send + 'static,
    ) -> Result<(Self, T)> {
        let (sender, ready) = mpsc::channel();
        let thread = thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                create_queue();
                // SAFETY: GetCurrentThreadId has no preconditions.
                let thread_id = unsafe { GetCurrentThreadId() };
                body(Ready { thread_id, sender });
            })?;
        let setup = ready.recv().unwrap_or_else(|_| {
            let reason = format!("the {name} thread ended before it was ready");
            Err(Error::platform("start a message thread", reason))
        });
        match setup {
            Ok((thread_id, value)) => {
                let thread = Self {
                    thread_id,
                    thread: Some(thread),
                };
                Ok((thread, value))
            }
            Err(e) => {
                let _ = thread.join();
                Err(e)
            }
        }
    }

    /// Posts `message` to the thread's queue.
    pub(crate) fn post(&self, message: u32, wparam: usize) -> Result<()> {
        // SAFETY: PostThreadMessageW only queues a message for the
        // thread, whose queue exists since spawn returned.
        unsafe { PostThreadMessageW(self.thread_id, message, WPARAM(wparam), LPARAM(0)) }
            .map_err(|e| Error::platform("post to a message thread", e))
    }

    /// Asks the thread to quit and waits until it has taken down what it
    /// owns and ended.
    pub(crate) fn stop(mut self) {
        self.quit();
    }

    fn quit(&mut self) {
        let Some(thread) = self.thread.take() else {
            return;
        };
        // The pump ends at WM_QUIT, after what was queued before it.
        // Posting fails while the queue is full, so it is tried again
        // until there is room or the thread has ended by itself. The open
        // JoinHandle keeps the thread's id from being reused meanwhile.
        while !thread.is_finished() && self.post(WM_QUIT, 0).is_err() {
            thread::sleep(QUIT_RETRY);
        }
        let _ = thread.join();
    }
}

/// Dropping it stops the thread, like [`MessageThread::stop`].
impl Drop for MessageThread {
    fn drop(&mut self) {
        self.quit();
    }
}

/// Dispatches the calling thread's messages until it receives `WM_QUIT`.
pub(crate) fn pump() {
    let mut msg = MSG::default();
    // SAFETY: GetMessageW blocks until a message arrives, writes it into
    // the stack allocated MSG, and returns false for WM_QUIT.
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
        // SAFETY: TranslateMessage and DispatchMessageW handle the message
        // just retrieved, calling the procedure of its window, if any.
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// Makes sure the calling thread has a message queue. Windows creates it
/// on the first call that needs one, and posting to a thread without one
/// fails.
fn create_queue() {
    let mut msg = MSG::default();
    // SAFETY: PeekMessageW with PM_NOREMOVE only looks at the calling
    // thread's queue, which creates it. The MSG is stack allocated.
    unsafe {
        let _ = PeekMessageW(&mut msg, None, WM_USER, WM_USER, PM_NOREMOVE);
    }
}

#[cfg(test)]
#[path = "message_thread_tests.rs"]
mod tests;
