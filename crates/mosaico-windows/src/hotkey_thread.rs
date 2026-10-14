//! The hotkey source: a thread that owns the global hotkeys.

use std::sync::mpsc::Sender;

use mosaico_core::config::Keybinding;
use mosaico_core::hotkey::{HotkeyControl, HotkeySource, HotkeyWatch, Hotkeys, Rejected};
use mosaico_core::{Action, Result};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_APP,
    WM_HOTKEY,
};

use crate::hotkey::{self, Win32Hotkeys};
use crate::message_thread::MessageThread;

/// `PostThreadMessageW` message for pause/unpause control.
/// wParam: 0 = unpause, 1 = pause.
const WM_HOTKEY_PAUSE: u32 = WM_APP + 2;

/// The global hotkeys of Windows, each start on a thread of its own.
///
/// `RegisterHotKey` without a window posts `WM_HOTKEY` to the thread
/// that registered the key, so the hotkeys are registered, dispatched,
/// paused and unregistered on that thread, and the watch reaches it with
/// thread messages. Nothing else is queued there, so a flood of window
/// events cannot hold a shortcut back.
pub(crate) struct WindowsHotkeySource;

impl HotkeySource for WindowsHotkeySource {
    fn start(&self, bindings: Vec<Keybinding>, actions: Sender<Action>) -> Result<HotkeyWatch> {
        let (thread, ()) = MessageThread::spawn("hotkeys", move |ready| {
            let (mut hotkeys, rejected) = Hotkeys::register(Win32Hotkeys::new(), &bindings);
            log_rejected(NOT_WORKING, &rejected);
            ready.report(Ok(()));
            run_message_pump(&mut hotkeys, &actions);
            // Dropping the hotkeys unregisters every one still registered.
        })?;
        Ok(HotkeyWatch::new(HotkeyThread(thread)))
    }
}

/// The thread a [`WindowsHotkeySource`] started.
struct HotkeyThread(MessageThread);

impl HotkeyControl for HotkeyThread {
    fn pause(&self) -> Result<()> {
        self.0.post(WM_HOTKEY_PAUSE, 1)
    }

    fn unpause(&self) -> Result<()> {
        self.0.post(WM_HOTKEY_PAUSE, 0)
    }

    /// Posts `WM_QUIT` after the requests already queued, and waits until
    /// the thread has unregistered the hotkeys and ended.
    fn stop(self: Box<Self>) {
        self.0.stop();
    }
}

/// The hotkey thread's message pump. Hotkey messages go first, so a
/// shortcut is handled before anything else waiting in the queue.
fn run_message_pump(hotkeys: &mut Hotkeys<Win32Hotkeys>, actions: &Sender<Action>) {
    let mut msg = MSG::default();

    loop {
        // Drain all pending WM_HOTKEY messages first.
        // SAFETY: PeekMessageW drains WM_HOTKEY messages from the thread's
        // message queue without blocking. The MSG struct is stack-allocated.
        while unsafe { PeekMessageW(&mut msg, None, WM_HOTKEY, WM_HOTKEY, PM_REMOVE).as_bool() } {
            dispatch(hotkeys, actions, msg.wParam.0);
        }

        // Block until the next message arrives, then process it.
        // SAFETY: GetMessageW blocks until a message arrives. Returns
        // false on WM_QUIT, signaling the pump to exit.
        if !unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
            break; // WM_QUIT
        }

        if msg.message == WM_HOTKEY {
            dispatch(hotkeys, actions, msg.wParam.0);
            continue;
        }

        if msg.message == WM_HOTKEY_PAUSE {
            match msg.wParam.0 {
                0 => log_rejected(NOT_WORKING, &hotkeys.unpause()),
                1 => log_rejected(STILL_HELD, &hotkeys.pause()),
                _ => {}
            }
            continue;
        }

        // SAFETY: TranslateMessage and DispatchMessageW process a single
        // message from the thread's queue, dispatching it to the procedure
        // of its window, if it has one.
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// Sends the action of the hotkey a `WM_HOTKEY` message reports, unless
/// the hotkeys say it should not run.
fn dispatch(hotkeys: &Hotkeys<Win32Hotkeys>, actions: &Sender<Action>, wparam: usize) {
    if let Some(action) = hotkey::hotkey_id(wparam).and_then(|id| hotkeys.triggered(id)) {
        let _ = actions.send(action);
    }
}

/// How the log introduces a binding that could not be registered.
const NOT_WORKING: &str = "Hotkey not working";

/// How the log introduces a hotkey a pause could not give back, which
/// stays registered and keeps running its action.
const STILL_HELD: &str = "Hotkey could not be released while paused";

fn log_rejected(what: &str, rejected: &[Rejected]) {
    for binding in rejected {
        mosaico_core::log_warn!("{what}: {binding}");
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{self, TryRecvError};

    use mosaico_core::hotkey::HotkeySource;

    use super::WindowsHotkeySource;

    #[test]
    fn stopping_the_watch_lets_go_of_the_action_sender() {
        // Arrange -- no bindings, so nothing is registered system-wide.
        let (actions, received) = mpsc::channel();
        let watch = WindowsHotkeySource
            .start(Vec::new(), actions)
            .expect("the thread starts");
        watch.pause().expect("the thread gets the pause");
        watch.unpause().expect("the thread gets the unpause");

        // Act
        watch.stop();

        // Assert -- the thread ended, so a loop over the actions ends too.
        assert_eq!(received.try_recv(), Err(TryRecvError::Disconnected));
    }
}
