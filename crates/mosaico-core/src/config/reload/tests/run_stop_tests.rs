use std::sync::mpsc;

use crate::Error;
use crate::config::reload::test_support::{GAP_7, temp_store, write_at};

use super::fake_watcher::{FakeWatcher, run_watch};
use super::*;

#[test]
fn a_stop_during_the_wait_wins_over_the_change() {
    // Arrange
    let store = temp_store("run-stop-wins");
    let config = store.config_path();
    let stop = AtomicBool::new(false);
    let changed_then_stopped = || {
        write_at(&config, GAP_7, 1);
        stop.store(true, Ordering::Relaxed);
    };
    let mut watcher =
        FakeWatcher::new(&stop).then(changed_then_stopped, Ok(DirectoryWake::Changed));

    // Act
    let (result, reloads) = run_watch(&store, &mut watcher, &stop);

    // Assert
    assert!(result.is_ok(), "{result:?}");
    assert!(reloads.is_empty(), "{reloads:?}");
    assert_eq!(watcher.timeouts.len(), 1);
}

#[test]
fn a_stop_set_before_the_watch_starts_waits_for_nothing() {
    // Arrange
    let store = temp_store("run-stopped");
    let stop = AtomicBool::new(true);
    let mut watcher = FakeWatcher::new(&stop);

    // Act
    let (result, _) = run_watch(&store, &mut watcher, &stop);

    // Assert
    assert!(result.is_ok(), "{result:?}");
    assert!(watcher.timeouts.is_empty(), "{:?}", watcher.timeouts);
}

#[test]
fn the_watch_ends_when_nothing_receives_the_reloads() {
    // Arrange
    let store = temp_store("run-receiver-gone");
    let config = store.config_path();
    let stop = AtomicBool::new(false);
    let mut watcher = FakeWatcher::new(&stop)
        .then(|| write_at(&config, GAP_7, 1), Ok(DirectoryWake::Changed))
        .then(|| write_at(&config, GAP_7, 2), Ok(DirectoryWake::Changed));
    let lent = &mut watcher;
    let (tx, rx) = mpsc::channel();
    drop(rx);

    // Act
    let result = ConfigWatch::open(store.clone(), |_| Ok(lent)).and_then(|w| w.run(tx, &stop));

    // Assert -- the first reload could not be sent, so it never waited
    // again.
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(watcher.timeouts.len(), 1);
}

#[test]
fn a_failed_wait_ends_the_watch_with_its_error() {
    // Arrange
    let store = temp_store("run-wait-failed");
    let stop = AtomicBool::new(false);
    let failed = Err(Error::platform("keep watching the directory", "it is gone"));
    let mut watcher = FakeWatcher::new(&stop).then(|| {}, failed);

    // Act
    let (result, reloads) = run_watch(&store, &mut watcher, &stop);

    // Assert
    assert!(
        matches!(
            result,
            Err(Error::Platform {
                operation: "keep watching the directory",
                ..
            })
        ),
        "{result:?}"
    );
    assert!(reloads.is_empty(), "{reloads:?}");
    assert_eq!(watcher.timeouts.len(), 1);
}
