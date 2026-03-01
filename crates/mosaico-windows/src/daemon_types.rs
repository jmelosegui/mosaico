use std::sync::mpsc;

use mosaico_core::Action;
use mosaico_core::ipc::{Command, Response};

use crate::config_watcher::ConfigReload;

/// Internal message type for the main daemon thread.
pub(super) enum DaemonMsg {
    /// A window event from the event loop.
    Event(mosaico_core::WindowEvent),
    /// A user action from hotkeys or IPC.
    Action(Action),
    /// A CLI command with a callback to send the response.
    Command(Command, ResponseSender),
    /// A validated config reload from the file watcher.
    Reload(Box<ConfigReload>),
    /// 1-second tick for refreshing bar system widgets.
    Tick,
}

/// Sends a response back to the IPC thread for the connected client.
pub(super) type ResponseSender = mpsc::Sender<Response>;
