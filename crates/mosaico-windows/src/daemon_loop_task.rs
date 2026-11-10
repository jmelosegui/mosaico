//! What the main loop runs as a task: every daemon message except a
//! window event, which the scheduler batches instead, and a display frame,
//! which it folds with the others of its wake into one frame step.

use mosaico_core::Action;
use mosaico_core::action::Origin;
use mosaico_core::bar::{BarHost, BarId, PointerEvent};
use mosaico_core::config::ConfigReload;
use mosaico_core::daemon::Message;
use mosaico_core::ipc::Command;
use mosaico_core::tray::TrayCommand;

use crate::tiling::TilingManager;

use super::daemon_inputs::Inputs;
use super::daemon_loop_commands::{self, carry_out};
use super::daemon_types::{DaemonMsg, ResponseSender};
use super::{daemon_loop_bar, daemon_loop_handlers, daemon_loop_tray};

/// A daemon message that is not a window event or a display frame.
pub(super) enum Task {
    /// A user action from hotkeys.
    Action(Action),
    /// A press of the launch binding at this position.
    Launch(usize),
    /// What the pointer did on a bar. It counts as input, like a key.
    BarPointer(BarId, PointerEvent),
    /// A CLI command with a callback to send the response.
    Command(Command, ResponseSender),
    /// What the user picked on the tray icon. It counts as input.
    Tray(TrayCommand),
    /// A validated config reload from the file watcher.
    Reload(Box<ConfigReload>),
    /// The newer version on offer, from the version check.
    Update(String),
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
        DaemonMsg::Launch(index) => Message::Task(Task::Launch(index)),
        DaemonMsg::BarPointer(bar, event) => Message::Task(Task::BarPointer(bar, event)),
        DaemonMsg::Command(command, reply) => Message::Task(Task::Command(command, reply)),
        DaemonMsg::Tray(command) => Message::Task(Task::Tray(command)),
        DaemonMsg::Reload(reload) => Message::Task(Task::Reload(reload)),
        DaemonMsg::Update(text) => Message::Task(Task::Update(text)),
        DaemonMsg::IpcFailed => Message::Task(Task::IpcFailed),
    }
}

/// Runs `task`, and says whether the daemon should stop: once a `Stop`
/// command has been answered, Quit was picked on the tray icon, or the
/// IPC listener died.
pub(super) fn run(
    task: Task,
    manager: &mut TilingManager,
    bars: &mut BarHost,
    inputs: &mut Inputs,
    hotkeys_paused: &mut bool,
) -> bool {
    match task {
        Task::Action(action) => {
            // Nobody waits for the answer of a hotkey, and `carry_out` has
            // logged a failure already.
            let origin = Origin::Focused;
            let _ = carry_out(action, origin, manager, bars, inputs, hotkeys_paused);
        }
        Task::Launch(index) => {
            daemon_loop_commands::launch(index, manager, bars, inputs);
        }
        Task::BarPointer(bar, event) => {
            daemon_loop_bar::handle_pointer(bar, event, manager, bars, inputs, hotkeys_paused);
        }
        Task::Command(command, reply_tx) => {
            if let Some(response) = daemon_loop_commands::handle_command(
                &command,
                manager,
                bars,
                inputs,
                hotkeys_paused,
            ) {
                let _ = reply_tx.send(response);
                return matches!(command, Command::Stop);
            }
        }
        Task::Tray(command) => {
            return daemon_loop_tray::handle(command, manager, bars, inputs, hotkeys_paused);
        }
        Task::Reload(reload) => daemon_loop_handlers::handle_reload(*reload, manager, bars, inputs),
        Task::Update(text) => {
            bars.set_update(&text);
            bars.show(manager);
        }
        Task::IpcFailed => return true,
    }
    false
}
