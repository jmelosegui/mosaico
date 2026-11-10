use std::sync::mpsc;

use mosaico_core::api::{self, API_VERSION};
use mosaico_core::bar::BarHost;
use mosaico_core::config::{ConfigStore, manage_deprecation};
use mosaico_core::ipc::DAEMON_VERSION;
use mosaico_core::tiler::HiddenFile;
use mosaico_core::tray::{TrayNotice, TrayPaths};
use mosaico_core::update::Installer;
use mosaico_core::{Instance, Result};

use crate::http::WinHttpClient;
use crate::install::RenameInstaller;
use crate::tiling::{TilingManager, Win32Platform};

use super::daemon_frames;
use super::daemon_inputs::{Inputs, Senders};
use super::daemon_loop_bar;
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
    let launches = store.load_launches();
    let rules = store.load_merged_rules();

    mosaico_core::log_info!(
        "Config: layout(gap={}, ratio={}), borders(width={}), log_level={}",
        config.layout.gap,
        config.layout.ratio,
        config.borders.width,
        config.logging.level
    );

    let (tx, rx) = mpsc::channel::<DaemonMsg>();

    // `bar.toml` is only read, and its new widgets added, while mosaico's
    // own bar is picked.
    let provider = store.bar_provider();
    if let Some(warning) = provider.warning() {
        mosaico_core::log_warn!("{warning}");
    }
    let daemon = api::Daemon {
        version: DAEMON_VERSION.to_string(),
        api: API_VERSION,
        instance: Instance::current().and_then(|i| i.name().map(str::to_string)),
        ..api::Daemon::default()
    };
    let mut bars = BarHost::new(
        &config.bar,
        daemon_loop_bar::make_builtin(tx.clone(), store.clone()),
        daemon,
        config.theme.resolve(),
    );
    mosaico_core::log_info!("Bar: {}", bars.choice().name());

    let hidden = Instance::current().map(HiddenFile::for_instance);
    let mut manager = TilingManager::new(Win32Platform::new(), hidden, &config, rules)?;
    // Retile clear of the bars, so borders match the final positions.
    bars.start(&mut manager);
    manager.refresh_border();

    // Background version check: runs once, and tells the loop of a newer
    // version.
    daemon_threads::spawn_version_check(WinHttpClient, tx.clone());

    // Background community-rules download — updates cached rules.toml.
    daemon_threads::spawn_rules_download(WinHttpClient, store.clone(), tx.clone());

    mosaico_core::log_info!("Managing {} windows", manager.window_count());

    // Start the window events, hotkeys, focus-follows-mouse and the tray
    // icon.
    let event_tx = tx.clone();
    let press_tx = tx.clone();
    let (event_channel_tx, event_channel_rx) = mpsc::channel();
    let (press_channel_tx, press_channel_rx) = mpsc::channel();
    let (tray_channel_tx, tray_channel_rx) = mpsc::channel();
    let senders = Senders {
        events: event_channel_tx,
        presses: press_channel_tx,
        tray: tray_channel_tx,
    };
    let mut inputs = Inputs::start(
        senders,
        keybindings,
        launches,
        &config,
        store.clone(),
        tray_paths(store),
    )?;
    if let Some(e) = inputs.sink_error() {
        mosaico_core::log_error!("Display and work area changes will not be noticed: {e}");
    }
    warn_of_manage_rules(store, &inputs);

    // Bridge: forward window events into the unified channel.
    let event_bridge = daemon_threads::spawn_event_bridge(event_channel_rx, event_tx);

    // Bridge: forward what hotkey presses ask for into the unified channel.
    let press_bridge = daemon_threads::spawn_press_bridge(press_channel_rx, press_tx);

    // Bridge: forward what the user picks on the tray icon.
    let tray_bridge = daemon_threads::spawn_tray_bridge(tray_channel_rx, tx.clone());

    // Start the IPC listener on its own thread.
    let ipc_thread = daemon_threads::spawn_ipc_listener(tx.clone(), pipe_name);

    // Start the config file watcher on its own thread.
    let (watcher_stop, watcher_thread, reload_bridge) =
        daemon_threads::spawn_config_watcher(tx.clone(), store.clone());

    // Display frames, sent while windows move. Without them, windows are
    // placed at once, and the daemon runs all the same.
    let frames = daemon_frames::start(tx.clone(), &mut manager);

    daemon_loop_run::run(&rx, &mut manager, &mut bars, &mut inputs, &frames);

    drop(frames);
    manager.restore_all_windows();
    bars.stop();
    inputs.stop();
    watcher_stop.store(true, std::sync::atomic::Ordering::Relaxed);
    drop(tx);
    let _ = event_bridge.join();
    let _ = press_bridge.join();
    let _ = tray_bridge.join();
    let _ = watcher_thread.join();
    let _ = reload_bridge.join();
    let _ = ipc_thread.join();

    Ok(())
}

/// Warns, in the log and beside the tray icon, when the user's rules
/// still use `manage`, which a future version will stop reading.
fn warn_of_manage_rules(store: &ConfigStore, inputs: &Inputs) {
    let count = store.user_rules_using_manage();
    if count == 0 {
        return;
    }
    mosaico_core::log_warn!("{}", manage_deprecation("user-rules.toml", count));
    inputs.notify(&TrayNotice::manage_deprecated(count));
}

/// What the tray icon opens: the settings app beside the running
/// executable, the config folder of `store` and the instance's log.
fn tray_paths(store: &ConfigStore) -> TrayPaths {
    let exe = std::env::current_exe().unwrap_or_default();
    TrayPaths {
        settings: exe.with_file_name(RenameInstaller.settings_name()),
        config_dir: store.dir().to_path_buf(),
        log: Instance::current()
            .map(Instance::log_path)
            .unwrap_or_default(),
    }
}
