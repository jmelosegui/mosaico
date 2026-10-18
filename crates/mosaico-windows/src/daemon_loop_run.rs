//! The daemon's main loop: waits for messages and runs what the
//! scheduler decides.
//!
//! [`Scheduler`] makes every timing decision: what runs first, which
//! window events are repeats, when a display change has settled, when
//! held effects are flushed, when moving windows step on and when the
//! wheel's turns on a bar are applied. This loop only waits, keeps the
//! thread's windows pumped, asks for frames while windows move, reads the
//! clock and carries out the steps it gets back.

use std::sync::mpsc;
use std::time::Instant;

use mosaico_core::WindowEvent;
use mosaico_core::bar::BarManager;
use mosaico_core::config::Theme;
use mosaico_core::daemon::{FrameClock, Scheduler, Step, Wake};

use crate::tiling::TilingManager;

use super::daemon_inputs::Inputs;
use super::daemon_loop_bar;
use super::daemon_loop_handlers;
use super::daemon_loop_pump::{PUMP_INTERVAL, pump_thread_messages};
use super::daemon_loop_task::{self, message};
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
        // state before doing anything else this iteration. This is also
        // when the bars report what the pointer did on them.
        pump_thread_messages();

        // Wait until something is due, in short slices instead of blocking
        // forever, so the message queue is drained on a regular cadence. The
        // thread is parked inside `recv_timeout` between wakeups, so an idle
        // daemon costs one wakeup per `PUMP_INTERVAL` and never spins.
        frames.want_frames(manager.wants_frames());
        let due = manager.frame_due(scheduler.last_frame_step());
        let wheel = bar_mgr.wheel_due();
        let wait = scheduler.begin_wait(manager.held_since(), due, wheel, Instant::now());
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
                Step::DisplayChanging => manager.display_changing(),
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
                Step::Run(task) => {
                    should_stop |= daemon_loop_task::run(
                        task,
                        manager,
                        bar_mgr,
                        current_theme,
                        inputs,
                        &mut hotkeys_paused,
                        get_update,
                    );
                }
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
                // The wheel switches workspaces once it has gone quiet over
                // a bar.
                Step::Wheel => daemon_loop_bar::apply_wheel(
                    manager,
                    bar_mgr,
                    inputs,
                    &mut hotkeys_paused,
                    get_update,
                ),
                Step::Frame => manager.frame(),
            }
        }
    }
}
