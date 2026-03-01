use std::sync::mpsc;

use mosaico_core::BspLayout;
use mosaico_core::WindowResult;
use mosaico_core::config;
use mosaico_core::ipc::Command;

use crate::bar_manager::BarManager;
use crate::event_loop;
use crate::monitor;
use crate::tiling::TilingManager;

use super::daemon_loop_handlers;
use super::daemon_threads;
use super::daemon_types::DaemonMsg;

/// The inner daemon loop, separated so cleanup always runs in `run()`.
pub(super) fn daemon_loop() -> WindowResult<()> {
    let config = config::load();
    mosaico_core::log::init(&config.logging);

    let keybindings = config::load_keybindings();
    let rules = config::load_merged_rules();

    mosaico_core::log_info!("Daemon started (PID: {})", std::process::id());
    mosaico_core::log_info!(
        "Config: layout(gap={}, ratio={}), borders(width={}), log_level={}",
        config.layout.gap,
        config.layout.ratio,
        config.borders.width,
        config.logging.level
    );

    let layout = BspLayout {
        gap: config.layout.gap,
        ratio: config.layout.ratio,
    };

    let mut current_theme = config.theme.resolve();

    let bar_config = config::load_bar();
    let monitor_rects: Vec<_> = monitor::enumerate_monitors()?
        .iter()
        .map(|m| m.work_area)
        .collect();
    let mut bar_mgr = BarManager::new(bar_config, monitor_rects, current_theme);

    let (tx, rx) = mpsc::channel::<DaemonMsg>();

    let mut manager = TilingManager::new(layout, rules, config.borders, config.layout.hiding)?;
    let bar_height = bar_mgr.bar_height();
    if bar_height > 0 {
        // Retile with bar-adjusted work areas so borders match final positions.
        manager.adjust_work_areas_for_bar(bar_height, bar_mgr.bar_monitor_indices());
    }
    manager.refresh_border();

    // Background version check — runs once, stores result for the bar.
    let update_text = daemon_threads::spawn_version_check();

    // Background community-rules download — updates cached rules.toml.
    daemon_threads::spawn_rules_download(tx.clone());

    let get_update = || update_text.lock().map_or(String::new(), |t| t.clone());
    bar_mgr.update(&manager.bar_states(&get_update()));
    mosaico_core::log_info!("Managing {} windows", manager.window_count());

    // Start the Win32 event loop + hotkeys on its own thread.
    let event_tx = tx.clone();
    let action_tx = tx.clone();
    let (event_channel_tx, event_channel_rx) = mpsc::channel();
    let (action_channel_tx, action_channel_rx) = mpsc::channel();
    let event_loop = event_loop::start(event_channel_tx, action_channel_tx, keybindings)?;

    // Bridge: forward window events into the unified channel.
    let event_bridge = daemon_threads::spawn_event_bridge(event_channel_rx, event_tx);

    // Bridge: forward hotkey actions into the unified channel.
    let action_bridge = daemon_threads::spawn_action_bridge(action_channel_rx, action_tx);

    // Start the IPC listener on its own thread.
    let ipc_thread = daemon_threads::spawn_ipc_listener(tx.clone());

    // Start the config file watcher on its own thread.
    let (watcher_stop, watcher_thread, reload_bridge) =
        daemon_threads::spawn_config_watcher(tx.clone());

    // 1-second tick for bar system widget refresh (clock, RAM).
    let tick_thread = daemon_threads::spawn_tick_thread(tx.clone(), watcher_stop.clone());

    // Main processing loop — blocks until a message arrives.
    while let Ok(msg) = rx.recv() {
        match msg {
            DaemonMsg::Event(event) => {
                daemon_loop_handlers::handle_event(
                    event,
                    &mut manager,
                    &mut bar_mgr,
                    &mut current_theme,
                    &get_update,
                );
            }
            DaemonMsg::Action(action) => {
                daemon_loop_handlers::handle_action(
                    action,
                    &mut manager,
                    &mut bar_mgr,
                    &get_update,
                );
            }
            DaemonMsg::Command(command, reply_tx) => {
                if let Some(response) = daemon_loop_handlers::handle_command(
                    &command,
                    &mut manager,
                    &mut bar_mgr,
                    &get_update,
                ) {
                    let _ = reply_tx.send(response);
                    if matches!(command, Command::Stop) {
                        break;
                    }
                }
            }
            DaemonMsg::Reload(reload) => {
                daemon_loop_handlers::handle_reload(
                    *reload,
                    &mut manager,
                    &mut bar_mgr,
                    &mut current_theme,
                    &get_update,
                );
            }
            DaemonMsg::Tick => {
                daemon_loop_handlers::handle_tick(&mut manager, &mut bar_mgr, &get_update);
            }
        }
    }

    manager.restore_all_windows();
    bar_mgr.hide_all();
    event_loop.stop();
    watcher_stop.store(true, std::sync::atomic::Ordering::Relaxed);
    drop(tx);
    let _ = event_bridge.join();
    let _ = action_bridge.join();
    let _ = watcher_thread.join();
    let _ = reload_bridge.join();
    let _ = tick_thread.join();
    let _ = ipc_thread.join();

    Ok(())
}
