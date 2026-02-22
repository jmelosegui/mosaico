//! Watches config files for changes and sends validated reloads.
//!
//! Uses `FindFirstChangeNotificationW` to monitor the config directory
//! for writes and renames. When a change is detected, mtimes are checked
//! to identify which file changed and only valid configs are sent.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::time::SystemTime;

use windows::Win32::Foundation::WAIT_OBJECT_0;
use windows::Win32::Storage::FileSystem::{
    FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_LAST_WRITE, FindCloseChangeNotification,
    FindFirstChangeNotificationW, FindNextChangeNotification,
};
use windows::Win32::System::Threading::WaitForSingleObject;
use windows::core::HSTRING;

use mosaico_core::config::{self, BarConfig, Config, WindowRule};

/// Timeout between stop-flag checks when no changes occur (ms).
const WAIT_TIMEOUT_MS: u32 = 5000;

/// A validated config reload ready to be applied.
pub enum ConfigReload {
    /// Layout and border settings changed.
    Config(Config),
    /// Window rules changed.
    Rules(Vec<WindowRule>),
    /// Bar configuration changed.
    Bar(Box<BarConfig>),
}

/// Runs the config watcher loop. Blocks until the stop flag is set
/// or the sender is dropped.
pub fn watch(tx: Sender<ConfigReload>, stop: Arc<AtomicBool>) {
    let Some(dir) = config::config_dir() else {
        mosaico_core::log_info!("config dir not found, watcher exiting");
        return;
    };

    let config_path = config::config_path();
    let rules_path = config::rules_path();
    let bar_path = config::bar_path();

    let mut config_mtime = mtime(config_path.as_deref());
    let mut rules_mtime = mtime(rules_path.as_deref());
    let mut bar_mtime = mtime(bar_path.as_deref());

    let dir_str = HSTRING::from(dir.as_os_str());
    let flags = FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_FILE_NAME;

    let handle = unsafe { FindFirstChangeNotificationW(&dir_str, false, flags) };
    let Ok(handle) = handle else {
        mosaico_core::log_info!("FindFirstChangeNotificationW failed, watcher exiting");
        return;
    };

    while !stop.load(Ordering::Relaxed) {
        let result = unsafe { WaitForSingleObject(handle, WAIT_TIMEOUT_MS) };
        if stop.load(Ordering::Relaxed) {
            break;
        }
        if result != WAIT_OBJECT_0 {
            continue; // timeout or error — loop back to check stop flag
        }

        if check_and_reload(
            &config_path,
            &mut config_mtime,
            &rules_path,
            &mut rules_mtime,
            &bar_path,
            &mut bar_mtime,
            &tx,
        ) {
            break; // sender dropped
        }

        let _ = unsafe { FindNextChangeNotification(handle) };
    }

    let _ = unsafe { FindCloseChangeNotification(handle) };
}

/// Checks mtimes and sends reloads for changed files.
/// Returns `true` if the sender has been dropped (caller should exit).
fn check_and_reload(
    config_path: &Option<std::path::PathBuf>,
    config_mtime: &mut Option<SystemTime>,
    rules_path: &Option<std::path::PathBuf>,
    rules_mtime: &mut Option<SystemTime>,
    bar_path: &Option<std::path::PathBuf>,
    bar_mtime: &mut Option<SystemTime>,
    tx: &Sender<ConfigReload>,
) -> bool {
    if let Some(path) = config_path {
        let new = mtime(Some(path.as_path()));
        if new != *config_mtime {
            *config_mtime = new;
            match config::try_load() {
                Ok(cfg) => {
                    mosaico_core::log_info!("config.toml changed, reloading");
                    if tx.send(ConfigReload::Config(cfg)).is_err() {
                        return true;
                    }
                }
                Err(e) => {
                    mosaico_core::log_info!("config.toml invalid, skipping: {e}");
                }
            }
        }
    }

    if let Some(path) = rules_path {
        let new = mtime(Some(path.as_path()));
        if new != *rules_mtime {
            *rules_mtime = new;
            match config::try_load_rules() {
                Ok(rules) => {
                    mosaico_core::log_info!("rules.toml changed, reloading");
                    if tx.send(ConfigReload::Rules(rules)).is_err() {
                        return true;
                    }
                }
                Err(e) => {
                    mosaico_core::log_info!("rules.toml invalid, skipping: {e}");
                }
            }
        }
    }

    if let Some(path) = bar_path {
        let new = mtime(Some(path.as_path()));
        if new != *bar_mtime {
            *bar_mtime = new;
            match config::try_load_bar() {
                Ok(bar) => {
                    mosaico_core::log_info!("bar.toml changed, reloading");
                    if tx.send(ConfigReload::Bar(Box::new(bar))).is_err() {
                        return true;
                    }
                }
                Err(e) => {
                    mosaico_core::log_info!("bar.toml invalid, skipping: {e}");
                }
            }
        }
    }

    false
}

/// Returns the modification time for a path, or `None` if unavailable.
fn mtime(path: Option<&std::path::Path>) -> Option<SystemTime> {
    path.and_then(|p| p.metadata().ok())
        .and_then(|m| m.modified().ok())
}
