//! The thread that owns the global hotkeys.

use std::sync::mpsc::Sender;

use mosaico_core::config::Keybinding;
use mosaico_core::{Action, Result};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage, WM_APP,
    WM_HOTKEY,
};

use crate::hotkey::HotkeyManager;
use crate::message_thread::MessageThread;

/// `PostThreadMessageW` message for pause/unpause control.
/// wParam: 0 = unpause, 1 = pause, 2 = toggle.
const WM_HOTKEY_PAUSE: u32 = WM_APP + 2;

/// The global hotkeys, on a thread of their own.
///
/// `RegisterHotKey` without a window posts `WM_HOTKEY` to the thread
/// that registered the key, so the hotkeys are registered, dispatched,
/// paused and unregistered on this thread, and the daemon reaches it
/// with thread messages. Nothing else is queued there, so a flood of
/// window events cannot hold a shortcut back.
pub(crate) struct HotkeyThread(MessageThread);

impl HotkeyThread {
    /// Registers `keybindings` on a new thread, which sends the action of
    /// each hotkey pressed to `actions`. A binding that cannot be
    /// registered is reported and skipped.
    pub(crate) fn start(actions: Sender<Action>, keybindings: Vec<Keybinding>) -> Result<Self> {
        let (thread, ()) = MessageThread::spawn("hotkeys", move |ready| {
            let mut hotkeys = HotkeyManager::new(actions);
            hotkeys.register_from_config(&keybindings);
            ready.report(Ok(()));
            run_message_pump(&mut hotkeys);
            // Dropping the manager unregisters every hotkey.
        })?;
        Ok(Self(thread))
    }

    /// Unregisters every hotkey except toggle-pause.
    pub(crate) fn pause(&self) {
        let _ = self.0.post(WM_HOTKEY_PAUSE, 1);
    }

    /// Registers the hotkeys that `pause` unregistered.
    pub(crate) fn unpause(&self) {
        let _ = self.0.post(WM_HOTKEY_PAUSE, 0);
    }

    /// Unregisters the hotkeys and waits for the thread to end.
    pub(crate) fn stop(self) {
        self.0.stop();
    }
}

/// The hotkey thread's message pump. Hotkey messages go first, so a
/// shortcut is handled before anything else waiting in the queue.
fn run_message_pump(hotkeys: &mut HotkeyManager) {
    let mut msg = MSG::default();

    loop {
        // Drain all pending WM_HOTKEY messages first.
        // SAFETY: PeekMessageW drains WM_HOTKEY messages from the thread's
        // message queue without blocking. The MSG struct is stack-allocated.
        while unsafe { PeekMessageW(&mut msg, None, WM_HOTKEY, WM_HOTKEY, PM_REMOVE).as_bool() } {
            hotkeys.dispatch(msg.wParam.0 as i32);
        }

        // Block until the next message arrives, then process it.
        // SAFETY: GetMessageW blocks until a message arrives. Returns
        // false on WM_QUIT, signaling the pump to exit.
        if !unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
            break; // WM_QUIT
        }

        if msg.message == WM_HOTKEY {
            hotkeys.dispatch(msg.wParam.0 as i32);
            continue;
        }

        if msg.message == WM_HOTKEY_PAUSE {
            match msg.wParam.0 {
                0 => hotkeys.unpause(),
                1 => hotkeys.pause(),
                _ => hotkeys.toggle_pause(),
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

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{self, TryRecvError};

    use super::HotkeyThread;

    #[test]
    fn stopping_the_thread_lets_go_of_the_action_sender() {
        // Arrange -- no bindings, so nothing is registered system-wide.
        // Pausing has nothing to show without a registered hotkey, so it
        // is not exercised here.
        let (actions, received) = mpsc::channel();
        let hotkeys = HotkeyThread::start(actions, Vec::new()).expect("the thread starts");

        // Act
        hotkeys.stop();

        // Assert -- the thread ended, so a loop over the actions ends too.
        assert_eq!(received.try_recv(), Err(TryRecvError::Disconnected));
    }
}
