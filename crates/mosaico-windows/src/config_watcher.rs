//! Polls config files for changes and sends validated reloads.
//!
//! Watches `config.toml` and `rules.toml` for modification time
//! changes. When a file changes, it validates the new content using
//! the `try_load` variants. Only valid configs are sent for reload.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::time::{Duration, SystemTime};

use mosaico_core::config::{self, Config, WindowRule};

/// Polling interval for checking file changes.
const POLL_INTERVAL: Duration = Duration::from_secs(2);

/// A validated config reload ready to be applied.
pub enum ConfigReload {
    /// Layout and border settings changed.
    Config(Config),
    /// Window rules changed.
    Rules(Vec<WindowRule>),
}

/// Runs the config watcher loop. Blocks until the stop flag is set
/// or the sender is dropped.
pub fn watch(tx: Sender<ConfigReload>, stop: Arc<AtomicBool>) {
    let config_path = config::config_path();
    let rules_path = config::rules_path();

    let mut config_mtime = mtime(config_path.as_deref());
    let mut rules_mtime = mtime(rules_path.as_deref());

    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(POLL_INTERVAL);

        if let Some(ref path) = config_path {
            let new_mtime = mtime(Some(path.as_path()));
            if new_mtime != config_mtime {
                config_mtime = new_mtime;
                match config::try_load() {
                    Ok(cfg) => {
                        mosaico_core::log_info!("config.toml changed, reloading");
                        if tx.send(ConfigReload::Config(cfg)).is_err() {
                            return;
                        }
                    }
                    Err(e) => {
                        mosaico_core::log_info!("config.toml changed but invalid, skipping: {e}");
                    }
                }
            }
        }

        if let Some(ref path) = rules_path {
            let new_mtime = mtime(Some(path.as_path()));
            if new_mtime != rules_mtime {
                rules_mtime = new_mtime;
                match config::try_load_rules() {
                    Ok(rules) => {
                        mosaico_core::log_info!("rules.toml changed, reloading");
                        if tx.send(ConfigReload::Rules(rules)).is_err() {
                            return;
                        }
                    }
                    Err(e) => {
                        mosaico_core::log_info!("rules.toml changed but invalid, skipping: {e}");
                    }
                }
            }
        }
    }
}

/// Returns the modification time for a path, or `None` if unavailable.
fn mtime(path: Option<&std::path::Path>) -> Option<SystemTime> {
    path.and_then(|p| p.metadata().ok())
        .and_then(|m| m.modified().ok())
}
