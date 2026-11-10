use mosaico_core::WindowEvent;
use mosaico_core::bar::BarHost;
use mosaico_core::config::ConfigReload;
use mosaico_core::tiler::Enumerator;
use mosaico_core::tray::TrayNotice;

use crate::enumerate::WindowsEnumerator;
use crate::tiling::TilingManager;

use super::daemon_inputs::Inputs;

/// Processes a single event. The bars are shown the state by the caller,
/// once per batch instead of after every event.
pub(super) fn handle_event(event: WindowEvent, manager: &mut TilingManager, bars: &mut BarHost) {
    match event {
        WindowEvent::WorkAreaChanged => {
            mosaico_core::log_info!("Work area changed (taskbar shown/hidden)");
            if let Err(e) = bars.reset_work_areas(manager) {
                mosaico_core::log_warn!("Work area changed, but {e}");
            }
        }
        WindowEvent::DisplayChanged => match WindowsEnumerator.monitors() {
            Ok(new_monitors) => {
                mosaico_core::log_info!("Display change detected, {} monitors", new_monitors.len());
                // The bars move first: the tiler keeps clear of where they
                // are *after* the change, so a monitor just connected is
                // not tiled underneath its bar.
                bars.display_changed(manager, new_monitors);
            }
            Err(e) => {
                mosaico_core::log_warn!("Failed to re-enumerate monitors: {}", e);
                manager.display_change_skipped();
            }
        },
        other => manager.handle_event(&other),
    }
}

pub(super) fn handle_reload(
    reload: ConfigReload,
    manager: &mut TilingManager,
    bars: &mut BarHost,
    inputs: &mut Inputs,
) {
    match reload {
        ConfigReload::Config(cfg) => {
            bars.set_theme(cfg.theme.resolve());
            manager.reload_config(&cfg);
            inputs.set_focus_follows_mouse(cfg.mouse.focus_follows_mouse);
            inputs.set_tray_enabled(cfg.tray.enabled);
        }
        ConfigReload::Rules(rules) => {
            manager.reload_rules(rules);
        }
        ConfigReload::Bar(bar_cfg) => {
            if let Err(e) = bars.reload(manager, *bar_cfg) {
                mosaico_core::log_warn!("Bar reloaded, but {e}");
            }
        }
        ConfigReload::Keybindings { bindings, launches } => {
            inputs.reload_hotkeys(bindings, launches);
        }
        ConfigReload::Invalid { file, reason } => {
            inputs.notify(&TrayNotice::invalid_config(file, &reason));
        }
    }
    bars.show(manager);
}

/// Once a second: the bars take in the update notice, then are shown the
/// state if it changed, or else redraw what changes on its own, such as the
/// clock.
pub(super) fn handle_tick(manager: &mut TilingManager, bars: &mut BarHost, update: &str) {
    bars.set_update(update);
    bars.tick(manager);
}
