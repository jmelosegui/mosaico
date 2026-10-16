use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::time::Duration;

use crate::Result;
use crate::config::ConfigStore;
use crate::watch::{DirectoryWake, DirectoryWatcher};

use super::{ConfigReload, ConfigReloader};

/// How long the watch waits for a change before it checks whether it was
/// asked to stop. Stopping the daemon waits up to this long for it.
const STOP_CHECK_INTERVAL: Duration = Duration::from_secs(5);

/// The config files of a store, reloaded as they change.
///
/// [`open`](Self::open) looks at the files before it starts watching their
/// directory. A change made while the watch starts is then reloaded with
/// the first change the watcher reports, rather than taken for how the
/// files started and missed until that file changes again.
#[derive(Debug)]
pub struct ConfigWatch<W> {
    reloader: ConfigReloader,
    watcher: W,
}

impl<W: DirectoryWatcher> ConfigWatch<W> {
    /// Records the watched files in `store`, then opens a watcher on their
    /// directory with `open`.
    ///
    /// # Errors
    ///
    /// Returns the error of `open`, when the directory cannot be watched.
    pub fn open(store: ConfigStore, open: impl FnOnce(&Path) -> Result<W>) -> Result<Self> {
        let reloader = ConfigReloader::new(store);
        let watcher = open(reloader.store.dir())?;
        Ok(Self { reloader, watcher })
    }

    /// Sends `reloads` a reload each time a watched file changes, until
    /// `stop` is set or nothing receives the reloads any more.
    ///
    /// Each time the watcher reports a change, the files are compared and
    /// loaded with [`ConfigReloader::reload_changed`]. While the directory
    /// is quiet, `stop` is checked every 5 seconds. A stop set while a
    /// change is reported wins: nothing is loaded.
    ///
    /// # Errors
    ///
    /// Returns the watcher's error when it can no longer watch the
    /// directory. Nothing is reloaded after that.
    pub fn run(mut self, reloads: Sender<ConfigReload>, stop: &AtomicBool) -> Result<()> {
        while !stop.load(Ordering::Relaxed) {
            let wake = self.watcher.wait(STOP_CHECK_INTERVAL)?;
            if stop.load(Ordering::Relaxed) {
                break;
            }
            if wake == DirectoryWake::TimedOut {
                continue;
            }
            for reload in self.reloader.reload_changed() {
                if reloads.send(reload).is_err() {
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/fake_watcher.rs"]
mod fake_watcher;

#[cfg(test)]
#[path = "tests/run_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/run_stop_tests.rs"]
mod stop_tests;
