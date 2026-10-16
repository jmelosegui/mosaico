//! The daemon's main loop: waits for messages and runs what the
//! scheduler decides.
//!
//! [`Scheduler`] makes every timing decision: what runs first, which
//! window events are repeats, when a display change has settled, and when
//! held effects are flushed and moving windows step on. This loop only
//! waits, keeps the thread's windows pumped, asks for frames while windows
//! move, reads the clock and carries out the steps it gets back.

use std::sync::mpsc;
use std::time::Instant;

use mosaico_core::WindowEvent;
use mosaico_core::bar::BarManager;
use mosaico_core::config::Theme;
use mosaico_core::daemon::{FrameClock, Scheduler, Step, Wake};
use mosaico_core::ipc::Command;

use crate::tiling::TilingManager;

use super::daemon_inputs::Inputs;
use super::daemon_loop_commands;
use super::daemon_loop_handlers;
use super::daemon_loop_pump::{PUMP_INTERVAL, pump_thread_messages};
use super::daemon_loop_task::{Task, message};
use super::daemon_types::DaemonMsg;

/// Runs daemon messages until a `Stop` command arrives, the IPC listener
/// fails or the channel closes.
pub(super) fn run(
    rx: &mpsc::Receiver<DaemonMsg>,
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    current_theme: &mut Theme,
    inputs: &mut Inputs,
    frames: &dyn FrameClock,
    get_update: &dyn Fn() -> String,
) {
    let mut scheduler = Scheduler::default();
    let mut should_stop = false;
    let mut hotkeys_paused = false;

    while !should_stop {
        // Keep the bar and border windows out of the not-responding
        // state before doing anything else this iteration.
        pump_thread_messages();

        // Wait until something is due, in short slices instead of
        // blocking forever, so the message queue is drained on a regular
        // cadence. The thread is parked inside `recv_timeout` between
        // wakeups, so an idle daemon costs one wakeup per `PUMP_INTERVAL`
        // and never spins. Display frames come while windows move.
        frames.want_frames(manager.wants_frames());
        let due = manager.frame_due(scheduler.last_frame_step());
        let wait = scheduler.begin_wait(manager.held_since(), due, Instant::now());
        let wait = wait.map_or(PUMP_INTERVAL, |left| left.min(PUMP_INTERVAL));
        let wake = match rx.recv_timeout(wait) {
            Ok(first) => Wake::Received(
                std::iter::once(first)
                    .chain(rx.try_iter())
                    .map(message)
                    .collect(),
            ),
            Err(mpsc::RecvTimeoutError::Timeout) => Wake::TimedOut,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };

        for step in scheduler.step(wake, Instant::now()) {
            match step {
                Step::Flush => manager.flush_pending(),
                Step::RebuildMonitors => {
                    if daemon_loop_handlers::handle_event(
                        WindowEvent::DisplayChanged,
                        manager,
                        bar_mgr,
                        current_theme,
                        get_update,
                    ) {
                        bar_mgr.update(&manager.bar_states(&get_update(), hotkeys_paused));
                    }
                }
                Step::Run(task) => match task {
                    Task::Action(action) => {
                        daemon_loop_commands::handle_action(
                            action,
                            manager,
                            bar_mgr,
                            inputs,
                            &mut hotkeys_paused,
                            get_update,
                        );
                    }
                    Task::Command(command, reply_tx) => {
                        if let Some(response) = daemon_loop_commands::handle_command(
                            &command,
                            manager,
                            bar_mgr,
                            inputs,
                            &mut hotkeys_paused,
                            get_update,
                        ) {
                            let _ = reply_tx.send(response);
                            if matches!(command, Command::Stop) {
                                should_stop = true;
                            }
                        }
                    }
                    Task::Reload(reload) => {
                        daemon_loop_handlers::handle_reload(
                            *reload,
                            manager,
                            bar_mgr,
                            current_theme,
                            inputs,
                            hotkeys_paused,
                            get_update,
                        );
                    }
                    Task::Tick => {
                        daemon_loop_handlers::handle_tick(
                            manager,
                            bar_mgr,
                            hotkeys_paused,
                            get_update,
                        );
                    }
                    Task::IpcFailed => should_stop = true,
                },
                Step::Events(events) => {
                    // Process the batch's window events, then refresh the
                    // bar once.
                    let mut needs_bar_update = false;
                    for event in events {
                        needs_bar_update |= daemon_loop_handlers::handle_event(
                            event,
                            manager,
                            bar_mgr,
                            current_theme,
                            get_update,
                        );
                    }
                    if needs_bar_update {
                        bar_mgr.update(&manager.bar_states(&get_update(), hotkeys_paused));
                    }
                }
                Step::Frame => manager.frame(),
            }
        }
    }
}
