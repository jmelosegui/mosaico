//! What the main loop runs as a task: every daemon message except a
//! window event, which the scheduler batches instead.

use mosaico_core::Action;
use mosaico_core::config::ConfigReload;
use mosaico_core::daemon::Message;
use mosaico_core::ipc::Command;

use super::daemon_types::{DaemonMsg, ResponseSender};

/// A daemon message that is not a window event.
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

/// Hands window events to the scheduler as events, to be batched, and
/// every other message as a task, to run ahead of them.
pub(super) fn message(msg: DaemonMsg) -> Message<Task> {
    match msg {
        DaemonMsg::Event(event) => Message::Event(event),
        DaemonMsg::Action(action) => Message::Task(Task::Action(action)),
        DaemonMsg::Command(command, reply) => Message::Task(Task::Command(command, reply)),
        DaemonMsg::Reload(reload) => Message::Task(Task::Reload(reload)),
        DaemonMsg::Tick => Message::Task(Task::Tick),
        DaemonMsg::IpcFailed => Message::Task(Task::IpcFailed),
    }
}
