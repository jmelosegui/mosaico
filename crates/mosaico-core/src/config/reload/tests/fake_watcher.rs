//! A scripted directory watcher, for the tests of the watch loop.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use crate::Result;
use crate::config::{ConfigReload, ConfigStore, ConfigWatch};
use crate::watch::{DirectoryWake, DirectoryWatcher};

/// One scripted wait: what happens while the watch waits, and how the
/// wait ends.
type Wait<'a> = (Box<dyn FnOnce() + 'a>, Result<DirectoryWake>);

/// A watcher that plays back scripted waits, then stops the watch the way
/// the daemon does when it shuts down.
pub(super) struct FakeWatcher<'a> {
    waits: VecDeque<Wait<'a>>,
    stop: &'a AtomicBool,
    /// The timeout of every wait, in order.
    pub(super) timeouts: Vec<Duration>,
}

impl<'a> FakeWatcher<'a> {
    pub(super) fn new(stop: &'a AtomicBool) -> Self {
        Self {
            waits: VecDeque::new(),
            stop,
            timeouts: Vec::new(),
        }
    }

    /// Adds a wait during which `during` happens, and that ends with `end`.
    pub(super) fn then(mut self, during: impl FnOnce() + 'a, end: Result<DirectoryWake>) -> Self {
        self.waits.push_back((Box::new(during), end));
        self
    }
}

impl DirectoryWatcher for FakeWatcher<'_> {
    fn wait(&mut self, timeout: Duration) -> Result<DirectoryWake> {
        self.timeouts.push(timeout);
        match self.waits.pop_front() {
            Some((during, end)) => {
                during();
                end
            }
            None => {
                self.stop.store(true, Ordering::Relaxed);
                Ok(DirectoryWake::TimedOut)
            }
        }
    }
}

/// Watches `store` with `watcher` until the watch ends, and returns how it
/// ended and what it sent.
pub(super) fn run_watch(
    store: &ConfigStore,
    watcher: &mut FakeWatcher<'_>,
    stop: &AtomicBool,
) -> (Result<()>, Vec<ConfigReload>) {
    let (tx, rx) = mpsc::channel();
    let result = ConfigWatch::open(store.clone(), move |_| Ok(watcher))
        .and_then(|watch| watch.run(tx, stop));
    (result, rx.try_iter().collect())
}
