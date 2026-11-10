//! What the user asks for: actions from hotkeys and commands from the
//! CLI.

use mosaico_core::action::Origin;
use mosaico_core::bar::BarHost;
use mosaico_core::ipc::{Command, DAEMON_VERSION, DaemonStatus, PROTOCOL_VERSION, Response};

use crate::tiling::{TilingManager, TilingViews};

use super::daemon_inputs::Inputs;

/// Carries out an action from a hotkey, the CLI or a bar, which `origin`
/// says, then shows the bars the state.
///
/// Fails only when something the action needed reported an error, which
/// is logged here, whoever sent the action. An action with nothing to act
/// on, such as focusing left at the left edge, does nothing and succeeds.
pub(super) fn carry_out(
    action: mosaico_core::Action,
    origin: Origin,
    manager: &mut TilingManager,
    bars: &mut BarHost,
    inputs: &mut Inputs,
    hotkeys_paused: &mut bool,
) -> mosaico_core::Result<()> {
    let done = if action == mosaico_core::Action::TogglePause {
        toggle_pause(inputs, hotkeys_paused);
        Ok(())
    } else {
        run_action(&action, origin, manager, bars)
    };
    if let Err(e) = &done {
        mosaico_core::log_warn!("{action}: {e}");
    }
    bars.set_paused(*hotkeys_paused);
    bars.show(manager);
    done
}

/// Carries out a press of the launch binding at `index`, tells how it
/// went, then shows the bars the state.
pub(super) fn launch(
    index: usize,
    manager: &mut TilingManager,
    bars: &mut BarHost,
    inputs: &Inputs,
) {
    let Some((binding, launcher)) = inputs.launch(index) else {
        mosaico_core::log_warn!("No launch binding at position {index}");
        return;
    };
    let outcome = manager.launch(binding, launcher);
    inputs.report_launch(binding, &outcome);
    bars.show(manager);
}

fn toggle_pause(inputs: &mut Inputs, hotkeys_paused: &mut bool) {
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
/// monitors cannot be read it still retiles, and reports why. The
/// retile itself then runs as the action, which also has every border
/// drawn from scratch.
fn run_action(
    action: &mosaico_core::Action,
    origin: Origin,
    manager: &mut TilingManager,
    bars: &mut BarHost,
) -> mosaico_core::Result<()> {
    if *action == mosaico_core::Action::Retile {
        mosaico_core::log_info!("Retile: re-reading work areas");
        let reread = bars.reset_work_areas(manager);
        manager.handle_action_from(action, origin);
        manager.refresh_visible_windows();
        return reread;
    }
    manager.handle_action_from(action, origin);
    Ok(())
}

pub(super) fn handle_command(
    command: &Command,
    manager: &mut TilingManager,
    bars: &mut BarHost,
    inputs: &mut Inputs,
    hotkeys_paused: &mut bool,
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
                bar: Some(bars.status()),
            };
            let msg = format!("Daemon is running, managing {windows} windows");
            Some(Response::ok_with_message(msg).with_data(&status))
        }
        Command::Action { action } => {
            let done = carry_out(
                *action,
                Origin::Focused,
                manager,
                bars,
                inputs,
                hotkeys_paused,
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
            bars.set_paused(*hotkeys_paused);
            bars.show(manager);
            Some(Response::ok_with_message("Hotkeys paused"))
        }
        Command::UnpauseHotkeys => {
            if *hotkeys_paused {
                inputs.unpause_hotkeys();
                *hotkeys_paused = false;
                mosaico_core::log_info!("Hotkeys unpaused via CLI");
            }
            bars.set_paused(*hotkeys_paused);
            bars.show(manager);
            Some(Response::ok_with_message("Hotkeys unpaused"))
        }
    }
}
