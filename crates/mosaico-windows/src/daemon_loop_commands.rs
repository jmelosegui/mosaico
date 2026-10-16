//! What the user asks for: actions from hotkeys and commands from the
//! CLI.

use mosaico_core::bar::BarManager;
use mosaico_core::ipc::{Command, DAEMON_VERSION, DaemonStatus, PROTOCOL_VERSION, Response};

use crate::tiling::TilingManager;

use super::daemon_inputs::Inputs;

/// Carries out an action from a hotkey. Nobody waits for an answer, and
/// [`carry_out`] has already logged a failure.
pub(super) fn handle_action(
    action: mosaico_core::Action,
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    inputs: &Inputs,
    hotkeys_paused: &mut bool,
    get_update: &dyn Fn() -> String,
) {
    let _ = carry_out(action, manager, bar_mgr, inputs, hotkeys_paused, get_update);
}

/// Carries out an action from a hotkey or the CLI, then refreshes the bar.
///
/// Fails only when something the action needed reported an error, which
/// is logged here, whoever sent the action. An action with nothing to act
/// on, such as focusing left at the left edge, does nothing and succeeds.
fn carry_out(
    action: mosaico_core::Action,
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    inputs: &Inputs,
    hotkeys_paused: &mut bool,
    get_update: &dyn Fn() -> String,
) -> mosaico_core::Result<()> {
    let done = if action == mosaico_core::Action::TogglePause {
        toggle_pause(inputs, hotkeys_paused);
        Ok(())
    } else {
        run_action(&action, manager, bar_mgr)
    };
    if let Err(e) = &done {
        mosaico_core::log_warn!("{action}: {e}");
    }
    bar_mgr.update(&manager.bar_states(&get_update(), *hotkeys_paused));
    done
}

fn toggle_pause(inputs: &Inputs, hotkeys_paused: &mut bool) {
    if *hotkeys_paused {
        inputs.unpause_hotkeys();
        *hotkeys_paused = false;
        mosaico_core::log_info!("Hotkeys unpaused");
    } else {
        inputs.pause_hotkeys();
        *hotkeys_paused = true;
        mosaico_core::log_info!("Hotkeys paused");
    }
}

/// Runs an action on the tiling manager.
///
/// Retile re-reads the OS work areas and re-reserves the bar before
/// laying windows out again, so it repairs a tiling area that went stale
/// (e.g. after a monitor reconnect) instead of re-applying it. When the
/// monitors cannot be read it still retiles, and reports why.
fn run_action(
    action: &mosaico_core::Action,
    manager: &mut TilingManager,
    bar_mgr: &BarManager,
) -> mosaico_core::Result<()> {
    if *action == mosaico_core::Action::Retile {
        mosaico_core::log_info!("Retile: re-reading work areas");
        let indices = bar_mgr.bar_monitor_indices().to_vec();
        let reread = manager.reset_and_adjust_work_areas(bar_mgr.bar_height(), &indices);
        manager.refresh_visible_windows();
        return reread;
    }
    manager.handle_action(action);
    Ok(())
}

pub(super) fn handle_command(
    command: &Command,
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    inputs: &Inputs,
    hotkeys_paused: &mut bool,
    get_update: &dyn Fn() -> String,
) -> Option<Response> {
    match command {
        Command::Stop => {
            mosaico_core::log_info!("Stop command received, shutting down");
            Some(Response::ok_with_message("Daemon stopping"))
        }
        Command::Status => {
            let windows = manager.window_count();
            let status = DaemonStatus {
                version: DAEMON_VERSION.to_string(),
                protocol: PROTOCOL_VERSION,
                windows,
            };
            let msg = format!("Daemon is running, managing {windows} windows");
            Some(Response::ok_with_message(msg).with_data(&status))
        }
        Command::Action { action } => {
            let done = carry_out(
                *action,
                manager,
                bar_mgr,
                inputs,
                hotkeys_paused,
                get_update,
            );
            Some(match done {
                Ok(()) => Response::ok(),
                Err(e) => Response::failed(&e),
            })
        }
        Command::Inspect => {
            // The text for people, the same state as data for programs.
            let snapshot = manager.snapshot();
            Some(Response::ok_with_message(snapshot.to_string()).with_data(&snapshot))
        }
        Command::PauseHotkeys => {
            if !*hotkeys_paused {
                inputs.pause_hotkeys();
                *hotkeys_paused = true;
                mosaico_core::log_info!("Hotkeys paused via CLI");
            }
            bar_mgr.update(&manager.bar_states(&get_update(), *hotkeys_paused));
            Some(Response::ok_with_message("Hotkeys paused"))
        }
        Command::UnpauseHotkeys => {
            if *hotkeys_paused {
                inputs.unpause_hotkeys();
                *hotkeys_paused = false;
                mosaico_core::log_info!("Hotkeys unpaused via CLI");
            }
            bar_mgr.update(&manager.bar_states(&get_update(), *hotkeys_paused));
            Some(Response::ok_with_message("Hotkeys unpaused"))
        }
    }
}
