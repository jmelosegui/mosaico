use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;

use mosaico_core::Action;
use mosaico_core::config::{ConfigReload, ConfigStore, ConfigWatch};
use mosaico_core::http::HttpClient;
use mosaico_core::{community_rules, update};

use crate::directory_watcher::WindowsDirectoryWatcher;

use super::daemon_ipc;
use super::daemon_types::DaemonMsg;

/// The folder of the mosaico-rules repository with the rules for Windows.
const COMMUNITY_RULE_SET: &str = "windows";

/// Spawns a background thread for the version check and returns the shared text.
///
/// The thread takes `http` with it, so the client must be `Send`.
pub(super) fn spawn_version_check(http: impl HttpClient + Send + 'static) -> Arc<Mutex<String>> {
    let update_text: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let update_text_writer = update_text.clone();
    thread::spawn(move || {
        if let Ok(Some(tag)) = update::newer_release(&http, env!("CARGO_PKG_VERSION"))
            && let Ok(mut text) = update_text_writer.lock()
        {
            *text = format!("{tag} available");
        }
    });
    update_text
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

/// Bridges hotkey actions into the daemon message channel.
pub(super) fn spawn_action_bridge(
    action_rx: mpsc::Receiver<Action>,
    tx: mpsc::Sender<DaemonMsg>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for action in action_rx {
            if tx.send(DaemonMsg::Action(action)).is_err() {
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
            mosaico_core::log_info!("FindFirstChangeNotificationW failed, watcher exiting: {e}");
            return;
        }
    };
    if let Err(e) = watch.run(reloads, stop) {
        mosaico_core::log_warn!("config watcher stopped: {e}");
    }
}

/// Spawns a 1-second tick thread for bar updates.
pub(super) fn spawn_tick_thread(
    tx: mpsc::Sender<DaemonMsg>,
    stop: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            thread::sleep(std::time::Duration::from_secs(1));
            if tx.send(DaemonMsg::Tick).is_err() {
                break;
            }
        }
    })
}
