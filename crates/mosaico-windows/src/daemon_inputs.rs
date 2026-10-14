use std::sync::mpsc::Sender;

use mosaico_core::config::Keybinding;
use mosaico_core::event::{EventSource, EventWatch, FocusFollowsMouse};
use mosaico_core::{Action, Error, Result, WindowEvent};

use crate::event_source::WindowsEventSource;
use crate::hotkey_thread::HotkeyThread;
use crate::pointer::WindowsPointer;

/// What the daemon listens to besides IPC: window events, hotkeys and
/// the pointer, each on a thread of its own.
pub(super) struct Inputs {
    events: EventWatch,
    hotkeys: HotkeyThread,
    focus_follows_mouse: FocusFollowsMouse,
}

impl Inputs {
    /// Starts sending window events to `events` and the actions of
    /// `keybindings` to `actions`, with focus-follows-mouse on or off.
    pub(super) fn start(
        events: Sender<WindowEvent>,
        actions: Sender<Action>,
        keybindings: Vec<Keybinding>,
        focus_follows_mouse: bool,
    ) -> Result<Self> {
        let watch = WindowsEventSource.start(events.clone())?;
        let hotkeys = HotkeyThread::start(actions, keybindings)?;
        let mut inputs = Self {
            events: watch,
            hotkeys,
            focus_follows_mouse: FocusFollowsMouse::new(WindowsPointer, events),
        };
        inputs.set_focus_follows_mouse(focus_follows_mouse);
        Ok(inputs)
    }

    /// Why display and work area changes will not be noticed, when the
    /// window that reports them could not be created.
    pub(super) fn sink_error(&self) -> Option<&Error> {
        self.events.display_error()
    }

    /// Unregisters every hotkey except toggle-pause.
    pub(super) fn pause_hotkeys(&self) {
        self.hotkeys.pause();
    }

    /// Registers the paused hotkeys again.
    pub(super) fn unpause_hotkeys(&self) {
        self.hotkeys.unpause();
    }

    /// Turns focus-follows-mouse on or off, as the config asks. When it
    /// cannot be turned on, the daemon goes on without it.
    pub(super) fn set_focus_follows_mouse(&mut self, enabled: bool) {
        if let Err(e) = self.focus_follows_mouse.set_enabled(enabled) {
            mosaico_core::log_warn!("Could not turn on focus-follows-mouse: {e}");
        }
    }

    /// Stops everything and waits for each thread to finish.
    pub(super) fn stop(self) {
        self.events.stop();
        self.hotkeys.stop();
        drop(self.focus_follows_mouse);
    }
}
