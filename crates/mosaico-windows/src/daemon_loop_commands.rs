//! What the user asks for: actions from hotkeys and commands from the
//! CLI.

use mosaico_core::ipc::{Command, DAEMON_VERSION, DaemonStatus, PROTOCOL_VERSION, Response};

use crate::bar_manager::BarManager;
use crate::tiling::TilingManager;

use super::daemon_inputs::Inputs;

pub(super) fn handle_action(
    action: mosaico_core::Action,
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    inputs: &Inputs,
    hotkeys_paused: &mut bool,
    get_update: &dyn Fn() -> String,
) {
    if action == mosaico_core::Action::TogglePause {
        if *hotkeys_paused {
            inputs.unpause_hotkeys();
            *hotkeys_paused = false;
            mosaico_core::log_info!("Hotkeys unpaused");
        } else {
            inputs.pause_hotkeys();
            *hotkeys_paused = true;
            mosaico_core::log_info!("Hotkeys paused");
        }
        bar_mgr.update(&manager.bar_states(&get_update(), *hotkeys_paused));
        return;
    }
    run_action(&action, manager, bar_mgr);
    bar_mgr.update(&manager.bar_states(&get_update(), *hotkeys_paused));
}

/// Runs an action on the tiling manager.
///
/// Retile re-reads the OS work areas and re-reserves the bar before
/// laying windows out again, so it repairs a tiling area that went stale
/// (e.g. after a monitor reconnect) instead of re-applying it.
fn run_action(action: &mosaico_core::Action, manager: &mut TilingManager, bar_mgr: &BarManager) {
    if *action == mosaico_core::Action::Retile {
        mosaico_core::log_info!("Retile: re-reading work areas");
        let indices = bar_mgr.bar_monitor_indices().to_vec();
        manager.reset_and_adjust_work_areas(bar_mgr.bar_height(), &indices);
        manager.refresh_visible_windows();
        return;
    }
    manager.handle_action(action);
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
            run_action(action, manager, bar_mgr);
            bar_mgr.update(&manager.bar_states(&get_update(), *hotkeys_paused));
            Some(Response::ok())
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
