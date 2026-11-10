use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::thread;

use mosaico_core::config::{ConfigReload, ConfigStore, ConfigWatch};
use mosaico_core::hotkey::Target;
use mosaico_core::http::HttpClient;
use mosaico_core::tray::TrayCommand;
use mosaico_core::{community_rules, update};

use crate::directory_watcher::WindowsDirectoryWatcher;

use super::daemon_ipc;
use super::daemon_types::DaemonMsg;

/// The folder of the mosaico-rules repository with the rules for Windows.
const COMMUNITY_RULE_SET: &str = "windows";

/// Spawns a background thread for the version check, which tells the
/// loop on `tx` once it finds a newer version. Nothing is sent otherwise.
///
/// The thread takes `http` with it, so the client must be `Send`.
pub(super) fn spawn_version_check(
    http: impl HttpClient + Send + 'static,
    tx: mpsc::Sender<DaemonMsg>,
) {
    thread::spawn(move || {
        if let Ok(Some(tag)) = update::newer_release(&http, env!("CARGO_PKG_VERSION")) {
            let _ = tx.send(DaemonMsg::Update(format!("{tag} available")));
        }
    });
}

/// Spawns a background thread that downloads the community rules to the
/// `rules.toml` of `store` and then reloads the merged rules.
///
/// The thread takes `http` with it, so the client must be `Send`.
pub(super) fn spawn_rules_download(
    http: impl HttpClient + Send + 'static,
    store: ConfigStore,
    tx: mpsc::Sender<DaemonMsg>,
) {
    thread::spawn(move || {
        if community_rules::download(&http, COMMUNITY_RULE_SET, &store.rules_path()).is_ok() {
            let rules = store.load_merged_rules();
            let _ = tx.send(DaemonMsg::Reload(Box::new(ConfigReload::Rules(rules))));
        }
    });
}

/// Bridges window events into the daemon message channel.
pub(super) fn spawn_event_bridge(
    event_rx: mpsc::Receiver<mosaico_core::WindowEvent>,
    tx: mpsc::Sender<DaemonMsg>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for event in event_rx {
            if tx.send(DaemonMsg::Event(event)).is_err() {
                break;
            }
        }
    })
}

/// Bridges what hotkey presses ask for into the daemon message channel.
pub(super) fn spawn_press_bridge(
    press_rx: mpsc::Receiver<Target>,
    tx: mpsc::Sender<DaemonMsg>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for target in press_rx {
            let msg = match target {
                Target::Action(action) => DaemonMsg::Action(action),
                Target::Launch(index) => DaemonMsg::Launch(index),
            };
            if tx.send(msg).is_err() {
                break;
            }
        }
    })
}

/// Bridges what the user picks on the tray icon into the daemon message
/// channel.
pub(super) fn spawn_tray_bridge(
    tray_rx: mpsc::Receiver<TrayCommand>,
    tx: mpsc::Sender<DaemonMsg>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for command in tray_rx {
            if tx.send(DaemonMsg::Tray(command)).is_err() {
                break;
            }
        }
    })
}

/// Spawns the IPC listener thread on `pipe_name`.
pub(super) fn spawn_ipc_listener(
    tx: mpsc::Sender<DaemonMsg>,
    pipe_name: &str,
) -> thread::JoinHandle<()> {
    let pipe_name = pipe_name.to_string();
    thread::spawn(move || daemon_ipc::ipc_loop(tx, pipe_name))
}

/// Spawns the watcher of the config files in `store` and a bridge into
/// the daemon channel.
pub(super) fn spawn_config_watcher(
    tx: mpsc::Sender<DaemonMsg>,
    store: ConfigStore,
) -> (
    Arc<AtomicBool>,
    thread::JoinHandle<()>,
    thread::JoinHandle<()>,
) {
    let (reload_tx, reload_rx) = mpsc::channel::<ConfigReload>();
    let watcher_stop = Arc::new(AtomicBool::new(false));
    let watcher_stop_flag = watcher_stop.clone();
    let watcher_thread = thread::spawn(move || watch_config(store, reload_tx, &watcher_stop_flag));

    let reload_bridge = thread::spawn(move || {
        for reload in reload_rx {
            if tx.send(DaemonMsg::Reload(Box::new(reload))).is_err() {
                break;
            }
        }
    });

    (watcher_stop, watcher_thread, reload_bridge)
}

/// Sends `reloads` the config files in `store` that change, until `stop`
/// is set.
fn watch_config(store: ConfigStore, reloads: mpsc::Sender<ConfigReload>, stop: &AtomicBool) {
    let watch = match ConfigWatch::open(store, WindowsDirectoryWatcher::new) {
        Ok(watch) => watch,
        Err(e) => {
            mosaico_core::log_warn!("config watcher could not start: {e}");
            return;
        }
    };
    if let Err(e) = watch.run(reloads, stop) {
        mosaico_core::log_warn!("config watcher stopped: {e}");
    }
}
