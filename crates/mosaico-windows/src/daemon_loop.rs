use std::sync::mpsc;

use mosaico_core::bar::BarManager;
use mosaico_core::config::ConfigStore;
use mosaico_core::tiler::{Enumerator, HiddenFile};
use mosaico_core::{Instance, Result};

use crate::bar::Win32BarRenderer;
use crate::enumerate::WindowsEnumerator;
use crate::http::WinHttpClient;
use crate::system_info::Win32SystemInfo;
use crate::tiling::{TilingManager, Win32Platform};

use super::daemon_frames;
use super::daemon_inputs::Inputs;
use super::daemon_loop_run;
use super::daemon_threads;
use super::daemon_types::DaemonMsg;

/// The inner daemon loop, separated so the cleanup in `lifecycle::run` always runs.
/// It reads and watches the config files in `store`.
pub(super) fn daemon_loop(pipe_name: &str, store: &ConfigStore) -> Result<()> {
    // Append any missing top-level config sections introduced by newer
    // versions of mosaico (e.g. [workspaces]) before loading, so the
    // user's config.toml visibly tracks the latest schema.
    store.merge_missing_config_sections();

    // Whatever was logged so far, including problems with the config,
    // waits for the logger, which needs the config to start.
    let config = store.load_config();
    mosaico_core::log::init(&config.logging);

    let keybindings = store.merge_missing_keybindings();
    let rules = store.load_merged_rules();

    mosaico_core::log_info!(
        "Config: layout(gap={}, ratio={}), borders(width={}), log_level={}",
        config.layout.gap,
        config.layout.ratio,
        config.borders.width,
        config.logging.level
    );

    let mut current_theme = config.theme.resolve();

    let bar_config = store.merge_missing_bar_widgets();
    let monitor_rects: Vec<_> = WindowsEnumerator
        .monitors()?
        .iter()
        .map(|m| m.work_area)
        .collect();
    let mut bar_mgr = BarManager::new(
        Box::new(Win32BarRenderer),
        Box::new(Win32SystemInfo),
        bar_config,
        monitor_rects,
        current_theme,
    );

    let (tx, rx) = mpsc::channel::<DaemonMsg>();

    let hidden = Instance::current().map(HiddenFile::for_instance);
    let mut manager = TilingManager::new(Win32Platform::new(), hidden, &config, rules)?;
    let bar_height = bar_mgr.bar_height();
    if bar_height > 0 {
        // Retile with bar-adjusted work areas so borders match final positions.
        manager.adjust_work_areas_for_bar(bar_height, bar_mgr.bar_monitor_indices());
    }
    manager.refresh_border();

    // Background version check — runs once, stores result for the bar.
    let update_text = daemon_threads::spawn_version_check(WinHttpClient);

    // Background community-rules download — updates cached rules.toml.
    daemon_threads::spawn_rules_download(WinHttpClient, store.clone(), tx.clone());

    let get_update = || update_text.lock().map_or(String::new(), |t| t.clone());
    bar_mgr.update(&manager.bar_states(&get_update(), false));
    mosaico_core::log_info!("Managing {} windows", manager.window_count());

    // Start the window events, hotkeys and focus-follows-mouse.
    let event_tx = tx.clone();
    let action_tx = tx.clone();
    let (event_channel_tx, event_channel_rx) = mpsc::channel();
    let (action_channel_tx, action_channel_rx) = mpsc::channel();
    let mut inputs = Inputs::start(
        event_channel_tx,
        action_channel_tx,
        keybindings,
        config.mouse.focus_follows_mouse,
    )?;
    if let Some(e) = inputs.sink_error() {
        mosaico_core::log_error!("Display and work area changes will not be noticed: {e}");
    }

    // Bridge: forward window events into the unified channel.
    let event_bridge = daemon_threads::spawn_event_bridge(event_channel_rx, event_tx);

    // Bridge: forward hotkey actions into the unified channel.
    let action_bridge = daemon_threads::spawn_action_bridge(action_channel_rx, action_tx);

    // Start the IPC listener on its own thread.
    let ipc_thread = daemon_threads::spawn_ipc_listener(tx.clone(), pipe_name);

    // Start the config file watcher on its own thread.
    let (watcher_stop, watcher_thread, reload_bridge) =
        daemon_threads::spawn_config_watcher(tx.clone(), store.clone());

    // 1-second tick for bar system widget refresh (clock, RAM).
    let tick_thread = daemon_threads::spawn_tick_thread(tx.clone(), watcher_stop.clone());

    // Display frames, sent while windows move. Without them, windows are
    // placed at once, and the daemon runs all the same.
    let frames = daemon_frames::start(tx.clone(), &mut manager);

    daemon_loop_run::run(
        &rx,
        &mut manager,
        &mut bar_mgr,
        &mut current_theme,
        &mut inputs,
        &frames,
        &get_update,
    );

    drop(frames);
    manager.restore_all_windows();
    bar_mgr.hide_all();
    inputs.stop();
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
