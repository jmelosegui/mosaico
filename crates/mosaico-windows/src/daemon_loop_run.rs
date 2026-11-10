//! The daemon's main loop: waits for messages and runs what the
//! scheduler decides.
//!
//! [`Scheduler`] makes every timing decision: what runs first, which
//! window events are repeats, when a display change has settled, when
//! held effects are flushed, when moving windows step on and when the
//! bars are woken. This loop only waits, keeps the
//! thread's windows pumped, asks for frames while windows move, reads the
//! clock and carries out the steps it gets back.

use std::sync::mpsc;
use std::time::Instant;

use mosaico_core::WindowEvent;
use mosaico_core::bar::BarHost;
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
    bars: &mut BarHost,
    inputs: &mut Inputs,
    frames: &dyn FrameClock,
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
        let bars_due = bars.due();
        let wait = scheduler.begin_wait(manager.held_since(), due, bars_due, Instant::now());
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
                    daemon_loop_handlers::handle_event(WindowEvent::DisplayChanged, manager, bars);
                    bars.show(manager);
                }
                Step::Run(task) => {
                    should_stop |=
                        daemon_loop_task::run(task, manager, bars, inputs, &mut hotkeys_paused);
                }
                Step::Events(events) => {
                    // Take in the batch's window events, then show the bars
                    // the state once.
                    for event in events {
                        daemon_loop_handlers::handle_event(event, manager, bars);
                    }
                    bars.show(manager);
                }
                // The wheel switches workspaces once it has gone quiet over
                // a bar, a clock turns, or a hold on a bar's space ends.
                Step::Bar => {
                    daemon_loop_bar::wake(manager, bars, inputs, &mut hotkeys_paused);
                }
                Step::Frame => manager.frame(),
            }
        }
    }
}
