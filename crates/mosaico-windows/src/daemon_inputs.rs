use std::sync::mpsc::Sender;

use mosaico_core::config::{ConfigReload, ConfigReloader, ConfigStore, Keybinding};
use mosaico_core::event::{EventSource, EventWatch, FocusFollowsMouse};
use mosaico_core::hotkey::{HotkeySource, HotkeyWatch};
use mosaico_core::tray::{Shortcuts, Tray, TrayCommand, TrayNotice, TrayPaths};
use mosaico_core::{Action, Config, Error, Result, WindowEvent};

use crate::event_source::WindowsEventSource;
use crate::files::WindowsFiles;
use crate::hotkey_thread::WindowsHotkeySource;
use crate::opener::ShellOpener;
use crate::pointer::WindowsPointer;
use crate::process::WindowsProcesses;
use crate::tray::WindowsTraySource;

/// Where the inputs send what they hear.
pub(super) struct Senders {
    /// Window events.
    pub(super) events: Sender<WindowEvent>,
    /// The actions of hotkeys.
    pub(super) actions: Sender<Action>,
    /// What the user picks on the tray icon.
    pub(super) tray: Sender<TrayCommand>,
}

/// What the daemon listens to besides IPC: window events, hotkeys, the
/// pointer and the tray icon, each on a thread of its own.
pub(super) struct Inputs {
    events: EventWatch,
    hotkeys: HotkeyWatch,
    focus_follows_mouse: FocusFollowsMouse,
    tray: Tray,
    tray_paths: TrayPaths,
    reloader: ConfigReloader,
}

impl Inputs {
    /// Starts sending window events, the actions of `keybindings` and
    /// what the user picks on the tray icon to `senders`, with
    /// focus-follows-mouse and the icon as `config` says. The icon's
    /// menu opens what `tray_paths` names and reloads the files of
    /// `store`.
    pub(super) fn start(
        senders: Senders,
        keybindings: Vec<Keybinding>,
        config: &Config,
        store: ConfigStore,
        tray_paths: TrayPaths,
    ) -> Result<Self> {
        let watch = WindowsEventSource.start(senders.events.clone())?;
        let hotkeys = WindowsHotkeySource.start(keybindings, senders.actions)?;
        let version = env!("CARGO_PKG_VERSION");
        let mut inputs = Self {
            events: watch,
            hotkeys,
            focus_follows_mouse: FocusFollowsMouse::new(WindowsPointer, senders.events),
            tray: Tray::new(WindowsTraySource, senders.tray, version),
            tray_paths,
            reloader: ConfigReloader::new(store),
        };
        inputs.set_focus_follows_mouse(config.mouse.focus_follows_mouse);
        inputs.set_tray_enabled(config.tray.enabled);
        Ok(inputs)
    }

    /// Why display and work area changes will not be noticed, when the
    /// window that reports them could not be created.
    pub(super) fn sink_error(&self) -> Option<&Error> {
        self.events.display_error()
    }

    /// Unregisters every hotkey except toggle-pause, and shows the tray
    /// icon paused.
    pub(super) fn pause_hotkeys(&mut self) {
        if let Err(e) = self.hotkeys.pause() {
            mosaico_core::log_warn!("Could not pause the hotkeys: {e}");
        }
        self.tray.set_paused(true);
    }

    /// Registers the paused hotkeys again, and shows the tray icon
    /// running.
    pub(super) fn unpause_hotkeys(&mut self) {
        if let Err(e) = self.hotkeys.unpause() {
            mosaico_core::log_warn!("Could not unpause the hotkeys: {e}");
        }
        self.tray.set_paused(false);
    }

    /// Turns focus-follows-mouse on or off, as the config asks. When it
    /// cannot be turned on, the daemon goes on without it.
    pub(super) fn set_focus_follows_mouse(&mut self, enabled: bool) {
        if let Err(e) = self.focus_follows_mouse.set_enabled(enabled) {
            mosaico_core::log_warn!("Could not turn on focus-follows-mouse: {e}");
        }
    }

    /// Shows the tray icon or removes it, as the config asks.
    pub(super) fn set_tray_enabled(&mut self, enabled: bool) {
        self.tray.set_enabled(enabled);
    }

    /// Shows `notice` beside the tray icon, when it is shown.
    pub(super) fn notify(&self, notice: &TrayNotice) {
        self.tray.notify(notice);
    }

    /// What the tray icon's menu opens, and how.
    pub(super) fn shortcuts(&self) -> Shortcuts<'_> {
        Shortcuts {
            paths: &self.tray_paths,
            files: &WindowsFiles,
            processes: &WindowsProcesses,
            opener: &ShellOpener,
        }
    }

    /// Loads every config file again, for the tray's reload entry.
    pub(super) fn reload_all(&mut self) -> Vec<ConfigReload> {
        self.reloader.load_all()
    }

    /// Stops everything and waits for each thread to finish.
    pub(super) fn stop(self) {
        self.events.stop();
        self.hotkeys.stop();
        drop(self.focus_follows_mouse);
        self.tray.stop();
    }
}
