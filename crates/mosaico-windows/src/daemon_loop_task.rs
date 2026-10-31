//! What the main loop runs as a task: every daemon message except a
//! window event, which the scheduler batches instead, and a display frame,
//! which it folds with the others of its wake into one frame step.

use mosaico_core::Action;
use mosaico_core::action::Origin;
use mosaico_core::bar::{BarId, BarManager, PointerEvent};
use mosaico_core::config::{ConfigReload, Theme};
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
    /// What the pointer did on a bar. It counts as input, like a key.
    BarPointer(BarId, PointerEvent),
    /// A CLI command with a callback to send the response.
    Command(Command, ResponseSender),
    /// What the user picked on the tray icon. It counts as input.
    Tray(TrayCommand),
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
        DaemonMsg::BarPointer(bar, event) => Message::Task(Task::BarPointer(bar, event)),
        DaemonMsg::Command(command, reply) => Message::Task(Task::Command(command, reply)),
        DaemonMsg::Tray(command) => Message::Task(Task::Tray(command)),
        DaemonMsg::Reload(reload) => Message::Task(Task::Reload(reload)),
        DaemonMsg::Tick => Message::Task(Task::Tick),
        DaemonMsg::IpcFailed => Message::Task(Task::IpcFailed),
    }
}

/// Runs `task`, and says whether the daemon should stop: once a `Stop`
/// command has been answered, Quit was picked on the tray icon, or the
/// IPC listener died.
pub(super) fn run(
    task: Task,
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    current_theme: &mut Theme,
    inputs: &mut Inputs,
    hotkeys_paused: &mut bool,
    get_update: &dyn Fn() -> String,
) -> bool {
    match task {
        Task::Action(action) => {
            // Nobody waits for the answer of a hotkey, and `carry_out` has
            // logged a failure already.
            let origin = Origin::Focused;
            let _ = carry_out(
                action,
                origin,
                manager,
                bar_mgr,
                inputs,
                hotkeys_paused,
                get_update,
            );
        }
        Task::BarPointer(bar, event) => {
            daemon_loop_bar::handle_pointer(
                bar,
                event,
                manager,
                bar_mgr,
                inputs,
                hotkeys_paused,
                get_update,
            );
        }
        Task::Command(command, reply_tx) => {
            if let Some(response) = daemon_loop_commands::handle_command(
                &command,
                manager,
                bar_mgr,
                inputs,
                hotkeys_paused,
                get_update,
            ) {
                let _ = reply_tx.send(response);
                return matches!(command, Command::Stop);
            }
        }
        Task::Tray(command) => {
            return daemon_loop_tray::handle(
                command,
                manager,
                bar_mgr,
                current_theme,
                inputs,
                hotkeys_paused,
                get_update,
            );
        }
        Task::Reload(reload) => daemon_loop_handlers::handle_reload(
            *reload,
            manager,
            bar_mgr,
            current_theme,
            inputs,
            *hotkeys_paused,
            get_update,
        ),
        Task::Tick => {
            daemon_loop_handlers::handle_tick(manager, bar_mgr, *hotkeys_paused, get_update);
        }
        Task::IpcFailed => return true,
    }
    false
}
