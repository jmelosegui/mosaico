use std::sync::mpsc;

use crate::Error;
use crate::config::reload::test_support::{GAP_7, kinds, temp_store, write_at};

use super::fake_watcher::{FakeWatcher, run_watch};
use super::*;

#[test]
fn each_change_the_watcher_reports_reloads_what_changed() {
    // Arrange
    let store = temp_store("run-changes");
    let (config, bar) = (store.config_path(), store.bar_path());
    let stop = AtomicBool::new(false);
    let mut watcher = FakeWatcher::new(&stop)
        .then(|| write_at(&config, GAP_7, 1), Ok(DirectoryWake::Changed))
        .then(|| write_at(&bar, "", 1), Ok(DirectoryWake::Changed));

    // Act
    let (result, reloads) = run_watch(&store, &mut watcher, &stop);

    // Assert
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(kinds(&reloads), ["config", "bar"]);
}

#[test]
fn a_change_made_while_the_watcher_opens_is_reloaded_with_the_next_change() {
    // Arrange
    let store = temp_store("run-open-race");
    let (config, bar) = (store.config_path(), store.bar_path());
    let stop = AtomicBool::new(false);
    let mut watcher =
        FakeWatcher::new(&stop).then(|| write_at(&bar, "", 1), Ok(DirectoryWake::Changed));
    let lent = &mut watcher;
    let (tx, rx) = mpsc::channel();

    // Act -- config.toml changes before the watcher is open, so the
    // watcher never reports it.
    let watch = ConfigWatch::open(store.clone(), |dir| {
        assert_eq!(dir, store.dir());
        write_at(&config, GAP_7, 1);
        Ok(lent)
    });
    let result = watch.and_then(|watch| watch.run(tx, &stop));

    // Assert -- the files were recorded before the watcher opened, so the
    // first change it reports brings config.toml along.
    assert!(result.is_ok(), "{result:?}");
    let reloads: Vec<_> = rx.try_iter().collect();
    assert_eq!(kinds(&reloads), ["config", "bar"]);
}

#[test]
fn a_watcher_that_cannot_be_opened_is_the_error() {
    // Arrange
    let store = temp_store("run-open-failed");

    // Act
    let result = ConfigWatch::<FakeWatcher<'_>>::open(store.clone(), |_| {
        Err(Error::platform("watch the directory", "it does not exist"))
    });

    // Assert
    assert!(
        matches!(
            result,
            Err(Error::Platform {
                operation: "watch the directory",
                ..
            })
        ),
        "{:?}",
        result.err()
    );
}

#[test]
fn a_config_written_after_starting_without_a_config_directory_is_reloaded() {
    // Arrange -- the daemon started before `mosaico init` made the config
    // directory, and the settings app saved config.toml into it later.
    let store = temp_store("run-no-dir");
    std::fs::remove_dir_all(store.dir()).unwrap();
    let config = store.config_path();
    let stop = AtomicBool::new(false);
    let mut watcher =
        FakeWatcher::new(&stop).then(|| write_at(&config, GAP_7, 1), Ok(DirectoryWake::Changed));
    let lent = &mut watcher;
    let (tx, rx) = mpsc::channel();

    // Act -- like every platform watcher, this one cannot watch a
    // directory that does not exist.
    let watch = ConfigWatch::open(store.clone(), |dir| {
        if dir.is_dir() {
            Ok(lent)
        } else {
            Err(Error::platform("watch the directory", "it does not exist"))
        }
    });
    let result = watch.and_then(|watch| watch.run(tx, &stop));

    // Assert
    assert!(result.is_ok(), "{result:?}");
    let reloads: Vec<_> = rx.try_iter().collect();
    assert_eq!(kinds(&reloads), ["config"]);
}

#[test]
fn a_wait_that_times_out_loads_nothing_even_when_a_file_changed() {
    // Arrange
    let store = temp_store("run-timed-out");
    let config = store.config_path();
    let stop = AtomicBool::new(false);
    let mut watcher =
        FakeWatcher::new(&stop).then(|| write_at(&config, GAP_7, 1), Ok(DirectoryWake::TimedOut));

    // Act
    let (result, reloads) = run_watch(&store, &mut watcher, &stop);

    // Assert -- the files are only compared when the watcher reports a
    // change.
    assert!(result.is_ok(), "{result:?}");
    assert!(reloads.is_empty(), "{reloads:?}");
}

#[test]
fn a_spurious_wake_up_reloads_nothing() {
    // Arrange
    let store = temp_store("run-spurious");
    write_at(&store.config_path(), GAP_7, 1);
    let stop = AtomicBool::new(false);
    let mut watcher = FakeWatcher::new(&stop).then(|| {}, Ok(DirectoryWake::Changed));

    // Act
    let (result, reloads) = run_watch(&store, &mut watcher, &stop);

    // Assert
    assert!(result.is_ok(), "{result:?}");
    assert!(reloads.is_empty(), "{reloads:?}");
}

#[test]
fn each_wait_lasts_at_most_five_seconds_so_a_stop_is_seen() {
    // Arrange
    let store = temp_store("run-stop-check");
    let stop = AtomicBool::new(false);
    let mut watcher = FakeWatcher::new(&stop)
        .then(|| {}, Ok(DirectoryWake::TimedOut))
        .then(|| {}, Ok(DirectoryWake::Changed));

    // Act
    let _ = run_watch(&store, &mut watcher, &stop);

    // Assert
    assert_eq!(watcher.timeouts, [Duration::from_secs(5); 3]);
}
