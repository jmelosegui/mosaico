use mosaico_core::ipc::{Command, DAEMON_VERSION, DaemonStatus, PROTOCOL_VERSION, Response};
use mosaico_core::tiler::Enumerator;

use crate::bar_manager::BarManager;
use crate::enumerate::WindowsEnumerator;
use crate::tiling::TilingManager;

use super::daemon_inputs::Inputs;

/// Processes a single event. Returns `true` if the bar should be refreshed.
///
/// Bar updates are deferred to the caller so the bar is only refreshed
/// once per batch instead of after every individual event.
pub(super) fn handle_event(
    event: mosaico_core::WindowEvent,
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    current_theme: &mut mosaico_core::config::Theme,
    _get_update: &dyn Fn() -> String,
) -> bool {
    match event {
        mosaico_core::WindowEvent::WorkAreaChanged => {
            mosaico_core::log_info!("Work area changed (taskbar shown/hidden)");
            let bar_height = bar_mgr.bar_height();
            let indices = bar_mgr.bar_monitor_indices().to_vec();
            manager.reset_and_adjust_work_areas(bar_height, &indices);
            true
        }
        mosaico_core::WindowEvent::DisplayChanged => match WindowsEnumerator.monitors() {
            Ok(new_monitors) => {
                mosaico_core::log_info!("Display change detected, {} monitors", new_monitors.len());
                // Rebuild the bars first: the tiler must reserve space on
                // the monitors that have a bar *after* the change. Reading
                // the indices beforehand leaves a newly connected monitor
                // tiled underneath its bar.
                let monitor_rects: Vec<_> = new_monitors.iter().map(|m| m.work_area).collect();
                bar_mgr.rebuild_for_monitors(monitor_rects, *current_theme);
                let bar_height = bar_mgr.bar_height();
                let indices = bar_mgr.bar_monitor_indices().to_vec();
                manager.handle_display_change(new_monitors, bar_height, &indices);
                true
            }
            Err(e) => {
                mosaico_core::log_warn!("Failed to re-enumerate monitors: {}", e);
                false
            }
        },
        other => {
            manager.handle_event(&other);
            // LocationChanged fires very frequently — only the
            // tiling manager needs it (for maximize detection).
            !matches!(other, mosaico_core::WindowEvent::LocationChanged { .. })
        }
    }
}

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
        Command::Inspect => Some(Response::ok_with_message(manager.inspect_state())),
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

pub(super) fn handle_reload(
    reload: crate::config_watcher::ConfigReload,
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    current_theme: &mut mosaico_core::config::Theme,
    inputs: &mut Inputs,
    hotkeys_paused: bool,
    get_update: &dyn Fn() -> String,
) {
    match reload {
        crate::config_watcher::ConfigReload::Config(cfg) => {
            *current_theme = cfg.theme.resolve();
            manager.reload_config(&cfg);
            inputs.set_focus_follows_mouse(cfg.mouse.focus_follows_mouse);
            // Theme may have changed — re-resolve bar colors.
            bar_mgr.resolve_colors(*current_theme);
            bar_mgr.update(&manager.bar_states(&get_update(), hotkeys_paused));
        }
        crate::config_watcher::ConfigReload::Rules(rules) => {
            manager.reload_rules(rules);
        }
        crate::config_watcher::ConfigReload::Bar(bar_cfg) => {
            let new_height = bar_mgr.reload(*bar_cfg);
            bar_mgr.resolve_colors(*current_theme);
            let indices = bar_mgr.bar_monitor_indices().to_vec();
            manager.reset_and_adjust_work_areas(new_height, &indices);
            bar_mgr.update(&manager.bar_states(&get_update(), hotkeys_paused));
        }
    }
}

pub(super) fn handle_tick(
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    hotkeys_paused: bool,
    get_update: &dyn Fn() -> String,
) {
    bar_mgr.update(&manager.bar_states(&get_update(), hotkeys_paused));
}
