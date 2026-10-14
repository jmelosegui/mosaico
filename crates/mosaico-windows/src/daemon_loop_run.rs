//! The daemon's main loop: waits for messages and runs them.

use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Instant;

use mosaico_core::config::Theme;
use mosaico_core::ipc::Command;

use crate::bar_manager::BarManager;
use crate::tiling::TilingManager;

use super::daemon_inputs::Inputs;
use super::daemon_loop_handlers;
use super::daemon_loop_pump::{PUMP_INTERVAL, pump_thread_messages};
use super::daemon_settle::DisplaySettle;
use super::daemon_types::DaemonMsg;

/// Runs daemon messages until a `Stop` command arrives, the IPC listener
/// fails or the channel closes.
pub(super) fn run(
    rx: &mpsc::Receiver<DaemonMsg>,
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    current_theme: &mut Theme,
    inputs: &mut Inputs,
    get_update: &dyn Fn() -> String,
) {
    // Main processing loop.
    //
    // Actions (hotkeys), commands, reloads, and ticks are always
    // processed before window events so that keyboard shortcuts
    // remain responsive even when the event queue is flooded.
    let mut events = Vec::new();
    let mut should_stop = false;
    let mut hotkeys_paused = false;
    let mut display_settle = DisplaySettle::default();

    while !should_stop {
        // Keep the bar and border windows out of the not-responding
        // state before doing anything else this iteration.
        pump_thread_messages();

        // Apply a display change once the configuration has gone quiet.
        if display_settle.take_due(Instant::now())
            && daemon_loop_handlers::handle_event(
                mosaico_core::WindowEvent::DisplayChanged,
                manager,
                bar_mgr,
                current_theme,
                get_update,
            )
        {
            bar_mgr.update(&manager.bar_states(&get_update(), hotkeys_paused));
        }

        // Block until at least one message arrives. If a foreground
        // change is pending from the previous iteration, only wait
        // briefly: if no new action arrives, flush the deferred
        // `SetForegroundWindow` and go back to a normal blocking
        // recv. This coalesces rapid focus navigation so the OS only
        // sees the *final* target while the border and bar icon keep
        // up with every keypress.
        let first = if manager.has_pending() {
            match rx.recv_timeout(std::time::Duration::from_millis(25)) {
                Ok(msg) => msg,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    manager.flush_pending();
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        } else {
            // Wait in short slices instead of blocking forever, so the
            // message queue is drained on a regular cadence. The thread is
            // parked inside `recv_timeout` between wakeups, so an idle
            // daemon costs one wakeup per `PUMP_INTERVAL` and never spins.
            let mut received = None;
            let mut disconnected = false;
            loop {
                let wait = display_settle
                    .remaining(Instant::now())
                    .map_or(PUMP_INTERVAL, |left| left.min(PUMP_INTERVAL));
                match rx.recv_timeout(wait) {
                    Ok(msg) => {
                        received = Some(msg);
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        pump_thread_messages();
                        if display_settle.is_due(Instant::now()) {
                            break;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }
            match received {
                Some(msg) => msg,
                None if disconnected => break,
                None => continue,
            }
        };

        // Drain all queued messages so we can prioritise.
        events.clear();
        let mut batch = vec![first];
        while let Ok(msg) = rx.try_recv() {
            batch.push(msg);
        }

        // Process priority messages (actions, commands, reloads, ticks)
        // first, deferring window events.
        for msg in batch {
            match msg {
                DaemonMsg::Event(event) => events.push(event),
                DaemonMsg::Action(action) => {
                    daemon_loop_handlers::handle_action(
                        action,
                        manager,
                        bar_mgr,
                        inputs,
                        &mut hotkeys_paused,
                        get_update,
                    );
                }
                DaemonMsg::Command(command, reply_tx) => {
                    if let Some(response) = daemon_loop_handlers::handle_command(
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
                DaemonMsg::Reload(reload) => {
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
                DaemonMsg::Tick => {
                    daemon_loop_handlers::handle_tick(manager, bar_mgr, hotkeys_paused, get_update);
                }
                DaemonMsg::IpcFailed => should_stop = true,
            }
        }

        // Deduplicate events before processing. LocationChanged and
        // TitleChanged fire very frequently; only the last per window
        // matters. This can collapse hundreds of events into a handful.
        dedup_events(&mut events);

        // Display changes are deferred until the configuration settles.
        let before = events.len();
        events.retain(|e| !matches!(e, mosaico_core::WindowEvent::DisplayChanged));
        if events.len() != before {
            display_settle.note(Instant::now());
        }

        // Process deferred window events, then refresh the bar once.
        let mut needs_bar_update = false;
        for event in events.drain(..) {
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

        // The deferred `SetForegroundWindow` for the final focus
        // target is flushed by the recv-timeout path at the top of
        // the next iteration, after a short quiet window. That way
        // rapid focus navigation coalesces across iterations, not
        // just within a single drained batch.
    }
}

/// Collapses duplicate high-frequency events in a batch.
///
/// `LocationChanged` and `TitleChanged` fire many times per second for the
/// same window. Only the last occurrence per window is meaningful — earlier
/// ones would just do redundant work. Other event types preserve order and
/// are never dropped because their side-effects are not idempotent.
fn dedup_events(events: &mut Vec<mosaico_core::WindowEvent>) {
    use mosaico_core::WindowEvent;

    if events.len() <= 1 {
        return;
    }

    // Track the last index of each dedupable (window, variant) pair.
    let mut last_seen: HashMap<(mosaico_core::WindowId, u8), usize> = HashMap::new();

    for (i, ev) in events.iter().enumerate() {
        let key = match ev {
            WindowEvent::LocationChanged { window } => (*window, 0u8),
            WindowEvent::TitleChanged { window } => (*window, 1),
            _ => continue, // non-dedupable
        };
        last_seen.insert(key, i);
    }

    if last_seen.is_empty() {
        return;
    }

    // Keep an event if it's non-dedupable OR the last of its kind.
    let mut i = 0;
    events.retain(|ev| {
        let dominated = match ev {
            WindowEvent::LocationChanged { window } => last_seen.get(&(*window, 0)) != Some(&i),
            WindowEvent::TitleChanged { window } => last_seen.get(&(*window, 1)) != Some(&i),
            _ => false,
        };
        i += 1;
        !dominated
    });
}
