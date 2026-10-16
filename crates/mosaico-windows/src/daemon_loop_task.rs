//! What the main loop runs as a task: every daemon message except a
//! window event, which the scheduler batches instead, and a display frame,
//! which it folds with the others of its wake into one frame step.

use mosaico_core::Action;
use mosaico_core::config::ConfigReload;
use mosaico_core::daemon::Message;
use mosaico_core::ipc::Command;

use super::daemon_types::{DaemonMsg, ResponseSender};

/// A daemon message that is not a window event or a display frame.
pub(super) enum Task {
    /// A user action from hotkeys.
    Action(Action),
    /// A CLI command with a callback to send the response.
    Command(Command, ResponseSender),
    /// A validated config reload from the file watcher.
    Reload(Box<ConfigReload>),
    /// 1-second tick for refreshing bar system widgets.
    Tick,
    /// The IPC listener died, so the daemon can no longer be controlled.
    IpcFailed,
}

/// Hands window events to the scheduler as events, to be batched, a
/// display frame as a frame, and every other message as a task, to run
/// ahead of them. Taking a frame drops its token, which lets the pacer
/// send the next one.
pub(super) fn message(msg: DaemonMsg) -> Message<Task> {
    match msg {
        DaemonMsg::Event(event) => Message::Event(event),
        DaemonMsg::Frame(_token) => Message::Frame,
        DaemonMsg::Action(action) => Message::Task(Task::Action(action)),
        DaemonMsg::Command(command, reply) => Message::Task(Task::Command(command, reply)),
        DaemonMsg::Reload(reload) => Message::Task(Task::Reload(reload)),
        DaemonMsg::Tick => Message::Task(Task::Tick),
        DaemonMsg::IpcFailed => Message::Task(Task::IpcFailed),
    }
}
