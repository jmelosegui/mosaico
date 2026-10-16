//! A window of the tests' own on a thread of its own, which handles its
//! messages when told to, or never.

use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use mosaico_core::WindowId;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::DestroyWindow;

use super::super::Window;
use super::super::test_windows::{Takes, create_child, create_taking};
use super::{pump, within_two_seconds};

#[path = "test_states.rs"]
mod states;

/// What the thread of a [`ThreadWindow`] is told.
enum Told {
    /// To make this change to its window, as its application would, and
    /// go on parking.
    Run(fn(HWND)),
    Pump,
    Stop,
}

/// A window owned by a thread of the test's own, which handles none of
/// its messages until told to: an application busy with something else,
/// or one of the hidden helper windows of Qt, Electron and WinForms,
/// which never do. Dropping it destroys the window on its thread, the
/// only thread that may, and waits for the thread to end.
pub(crate) struct ThreadWindow {
    raw: usize,
    told: mpsc::Sender<Told>,
    thread: Option<JoinHandle<()>>,
}

impl ThreadWindow {
    /// `None` when a window cannot be created at all, so the test skips
    /// rather than fails on a machine with no interactive desktop.
    pub(crate) fn new() -> Option<Self> {
        Self::taking(Takes::Moves)
    }

    /// A window as `new` makes, which takes its moves as `takes`.
    pub(crate) fn taking(takes: Takes) -> Option<Self> {
        Self::spawn(move || create_taking(takes).map(|hwnd| (hwnd, hwnd)))
    }

    /// A window as `new` makes, inside a hidden one, with `WS_VISIBLE`
    /// when `shown`: however it is shown, maximized or minimized, it never
    /// appears, as its parent never does.
    pub(crate) fn child(shown: bool) -> Option<Self> {
        Self::spawn(move || create_child(shown))
    }

    /// Runs `create` on a thread of its own, which parks with the window
    /// it gives the test, and destroys the other, its own or its parent,
    /// once told to stop.
    fn spawn(create: impl FnOnce() -> Option<(HWND, HWND)> + Send + 'static) -> Option<Self> {
        let (ready_tx, ready_rx) = mpsc::channel::<usize>();
        let (told, told_rx) = mpsc::channel();
        let thread = thread::spawn(move || {
            let created = create();
            let _ = ready_tx.send(created.map_or(0, |(_, hwnd)| hwnd.0 as usize));
            // Park without ever calling GetMessage or PeekMessage, until
            // told otherwise.
            loop {
                match told_rx.recv() {
                    Ok(Told::Run(change)) => created.into_iter().for_each(|(_, hwnd)| change(hwnd)),
                    Ok(Told::Pump) => {
                        let tick = Duration::from_millis(1);
                        while let Err(RecvTimeoutError::Timeout) = told_rx.recv_timeout(tick) {
                            pump();
                        }
                        break;
                    }
                    _ => break,
                }
            }
            if let Some((destroyed, _)) = created {
                // SAFETY: destroying a window this thread created, with
                // the window it gave the test when that is its child.
                unsafe {
                    let _ = DestroyWindow(destroyed);
                }
            }
        });
        let mut window = Self {
            raw: 0,
            told,
            thread: Some(thread),
        };
        match ready_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(raw) if raw != 0 => {
                window.raw = raw;
                Some(window)
            }
            _ => None,
        }
    }

    pub(crate) fn id(&self) -> WindowId {
        WindowId::from_raw(self.raw as u64)
    }

    /// Tells the thread to handle its messages from now on, every
    /// millisecond, until the window is dropped.
    pub(crate) fn pump(&self) {
        let _ = self.told.send(Told::Pump);
    }

    /// Tells the parked thread to give the window the maximized style, as
    /// its application would, without handling any of its messages, and
    /// says whether it has within two seconds.
    pub(crate) fn maximize(&self) -> bool {
        self.make(states::maximize, Window::is_maximized)
    }

    /// Tells the parked thread to minimize the window, as `maximize` does.
    pub(crate) fn minimize(&self) -> bool {
        self.make(states::minimize, Window::is_minimized)
    }

    /// Tells the parked thread to make `change` to the window, and says
    /// whether it is `done` within two seconds.
    fn make(&self, change: fn(HWND), done: fn(&Window) -> bool) -> bool {
        let _ = self.told.send(Told::Run(change));
        within_two_seconds(|| done(&Window::from_id(self.id())))
    }
}

impl Drop for ThreadWindow {
    fn drop(&mut self) {
        let _ = self.told.send(Told::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
