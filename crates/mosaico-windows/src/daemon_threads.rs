use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;

use mosaico_core::Action;

use crate::config_watcher::ConfigReload;

use super::daemon_ipc;
use super::daemon_types::DaemonMsg;

/// Spawns a background thread for the version check and returns the shared text.
pub(super) fn spawn_version_check() -> Arc<Mutex<String>> {
    let update_text: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let update_text_writer = update_text.clone();
    thread::spawn(move || {
        if let Some(tag) = crate::version_check::check_for_update()
            && let Ok(mut text) = update_text_writer.lock()
        {
            *text = format!("{tag} available");
        }
    });
    update_text
}

/// Spawns a background thread to download community rules.
pub(super) fn spawn_rules_download(tx: mpsc::Sender<DaemonMsg>) {
    thread::spawn(move || {
        if let Some(rules) = crate::community_rules::download() {
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

/// Spawns the IPC listener thread.
pub(super) fn spawn_ipc_listener(tx: mpsc::Sender<DaemonMsg>) -> thread::JoinHandle<()> {
    thread::spawn(move || daemon_ipc::ipc_loop(tx))
}

/// Spawns the config watcher thread and a bridge into the daemon channel.
pub(super) fn spawn_config_watcher(
    tx: mpsc::Sender<DaemonMsg>,
) -> (
    Arc<AtomicBool>,
    thread::JoinHandle<()>,
    thread::JoinHandle<()>,
) {
    let (reload_tx, reload_rx) = mpsc::channel::<ConfigReload>();
    let watcher_stop = Arc::new(AtomicBool::new(false));
    let watcher_stop_flag = watcher_stop.clone();
    let watcher_thread =
        thread::spawn(move || crate::config_watcher::watch(reload_tx, watcher_stop_flag));

    let reload_bridge = thread::spawn(move || {
        for reload in reload_rx {
            if tx.send(DaemonMsg::Reload(Box::new(reload))).is_err() {
                break;
            }
        }
    });

    (watcher_stop, watcher_thread, reload_bridge)
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
