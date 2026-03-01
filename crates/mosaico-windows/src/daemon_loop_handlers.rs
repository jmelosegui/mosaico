use mosaico_core::ipc::{Command, Response};

use crate::bar_manager::BarManager;
use crate::monitor;
use crate::tiling::TilingManager;

pub(super) fn handle_event(
    event: mosaico_core::WindowEvent,
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    current_theme: &mut mosaico_core::config::Theme,
    get_update: &dyn Fn() -> String,
) {
    match event {
        mosaico_core::WindowEvent::WorkAreaChanged => {
            mosaico_core::log_info!("Work area changed (taskbar shown/hidden)");
            let bar_height = bar_mgr.bar_height();
            let indices = bar_mgr.bar_monitor_indices().to_vec();
            manager.reset_and_adjust_work_areas(bar_height, &indices);
            bar_mgr.update(&manager.bar_states(&get_update()));
        }
        mosaico_core::WindowEvent::DisplayChanged => match monitor::enumerate_monitors() {
            Ok(new_monitors) => {
                mosaico_core::log_info!("Display change detected, {} monitors", new_monitors.len());
                let bar_height = bar_mgr.bar_height();
                let indices = bar_mgr.bar_monitor_indices().to_vec();
                manager.handle_display_change(new_monitors, bar_height, &indices);

                let monitor_rects: Vec<_> = monitor::enumerate_monitors()
                    .unwrap_or_default()
                    .iter()
                    .map(|m| m.work_area)
                    .collect();
                bar_mgr.rebuild_for_monitors(monitor_rects, *current_theme);
                bar_mgr.update(&manager.bar_states(&get_update()));
            }
            Err(e) => {
                mosaico_core::log_info!("Failed to re-enumerate monitors: {}", e);
            }
        },
        other => {
            manager.handle_event(&other);
            // LocationChanged fires very frequently — only the
            // tiling manager needs it (for maximize detection).
            // Skip bar updates to avoid unnecessary redraws.
            if !matches!(other, mosaico_core::WindowEvent::LocationChanged { .. }) {
                bar_mgr.update(&manager.bar_states(&get_update()));
            }
        }
    }
}

pub(super) fn handle_action(
    action: mosaico_core::Action,
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    get_update: &dyn Fn() -> String,
) {
    manager.handle_action(&action);
    bar_mgr.update(&manager.bar_states(&get_update()));
}

pub(super) fn handle_command(
    command: &Command,
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    get_update: &dyn Fn() -> String,
) -> Option<Response> {
    match command {
        Command::Stop => {
            mosaico_core::log_info!("Stop command received, shutting down");
            Some(Response::ok_with_message("Daemon stopping"))
        }
        Command::Status => {
            let msg = format!(
                "Daemon is running, managing {} windows",
                manager.window_count()
            );
            Some(Response::ok_with_message(msg))
        }
        Command::Action { action } => {
            manager.handle_action(action);
            bar_mgr.update(&manager.bar_states(&get_update()));
            Some(Response::ok())
        }
    }
}

pub(super) fn handle_reload(
    reload: crate::config_watcher::ConfigReload,
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    current_theme: &mut mosaico_core::config::Theme,
    event_loop: &crate::event_loop::EventLoopHandle,
    get_update: &dyn Fn() -> String,
) {
    match reload {
        crate::config_watcher::ConfigReload::Config(cfg) => {
            *current_theme = cfg.theme.resolve();
            manager.reload_config(&cfg);
            event_loop.toggle_focus_follows_mouse(cfg.mouse.focus_follows_mouse);
            // Theme may have changed — re-resolve bar colors.
            bar_mgr.resolve_colors(*current_theme);
            bar_mgr.update(&manager.bar_states(&get_update()));
        }
        crate::config_watcher::ConfigReload::Rules(rules) => {
            manager.reload_rules(rules);
        }
        crate::config_watcher::ConfigReload::Bar(bar_cfg) => {
            let new_height = bar_mgr.reload(*bar_cfg);
            bar_mgr.resolve_colors(*current_theme);
            let indices = bar_mgr.bar_monitor_indices().to_vec();
            manager.reset_and_adjust_work_areas(new_height, &indices);
            bar_mgr.update(&manager.bar_states(&get_update()));
        }
    }
}

pub(super) fn handle_tick(
    manager: &mut TilingManager,
    bar_mgr: &mut BarManager,
    get_update: &dyn Fn() -> String,
) {
    bar_mgr.update(&manager.bar_states(&get_update()));
}
