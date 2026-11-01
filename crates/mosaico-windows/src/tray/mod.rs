//! The mosaico icon in the notification area, on a thread of its own.
//!
//! The icon, its hidden window and its menu belong to the thread that
//! made them, so the watch reaches that thread through a channel and
//! wakes it with a message to the window. A message to the window, not
//! to the thread, is dispatched even while the menu is open.

mod decode;
mod icon;
mod menu;
mod window;

use std::sync::mpsc::{self, Sender};

use mosaico_core::tray::{TrayCommand, TrayControl, TrayNotice, TraySource, TrayState, TrayWatch};
use mosaico_core::{Error, Result};

use crate::message_thread::{self, MessageThread};

use window::{Request, TrayWindow, Waker};

/// The notification area icon of Windows.
pub struct WindowsTraySource;

impl TraySource for WindowsTraySource {
    fn start(&self, state: &TrayState, commands: Sender<TrayCommand>) -> Result<TrayWatch> {
        let (requests, inbox) = mpsc::channel();
        let state = state.clone();
        let (thread, waker) = MessageThread::spawn("tray", move |ready| {
            match TrayWindow::open(state, commands, inbox) {
                Ok(waker) => {
                    ready.report(Ok(waker));
                    message_thread::pump();
                    // Removes the icon and drops the sender.
                    window::close();
                }
                Err(e) => ready.report(Err(e)),
            }
        })?;
        Ok(TrayWatch::new(TrayThread {
            thread,
            waker,
            requests,
        }))
    }
}

/// The thread a [`WindowsTraySource`] started.
struct TrayThread {
    thread: MessageThread,
    waker: Waker,
    requests: Sender<Request>,
}

impl TrayThread {
    fn send(&self, request: Request) -> Result<()> {
        self.requests
            .send(request)
            .map_err(|_| Error::platform("reach the tray icon", "its thread has ended"))?;
        self.waker.wake()
    }
}

impl TrayControl for TrayThread {
    fn update(&self, state: &TrayState) -> Result<()> {
        self.send(Request::Update(state.clone()))
    }

    fn notify(&self, notice: &TrayNotice) -> Result<()> {
        self.send(Request::Notify(notice.clone()))
    }

    /// Posts `WM_QUIT` after the requests already queued, and waits until
    /// the thread has removed the icon and ended.
    fn stop(self: Box<Self>) {
        self.thread.stop();
    }
}
