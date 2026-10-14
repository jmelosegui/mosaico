use std::path::PathBuf;

use mosaico_core::Error;

use super::*;

/// Long enough for a change that has happened to be reported, however
/// loaded the machine is. A wait that sees one returns at once.
const LONG: Duration = Duration::from_secs(10);

/// Long enough for a change that is reported to arrive.
const SHORT: Duration = Duration::from_millis(200);

/// A fresh, empty directory named after `test`.
fn temp_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("mosaico-directory-watcher-tests")
        .join(format!("{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_write_in_the_directory_wakes_the_watcher() {
    // Arrange
    let dir = temp_dir("write");
    let mut watcher = WindowsDirectoryWatcher::new(&dir).unwrap();

    // Act
    std::fs::write(dir.join("config.toml"), "a = 1").unwrap();
    let wake = watcher.wait(LONG).unwrap();

    // Assert
    assert_eq!(wake, DirectoryWake::Changed);
}

#[test]
fn a_quiet_directory_times_out() {
    // Arrange
    let dir = temp_dir("quiet");
    let mut watcher = WindowsDirectoryWatcher::new(&dir).unwrap();

    // Act
    let wake = watcher.wait(SHORT).unwrap();

    // Assert
    assert_eq!(wake, DirectoryWake::TimedOut);
}

#[test]
fn a_reported_change_is_not_reported_again() {
    // Arrange
    let dir = temp_dir("reported");
    let mut watcher = WindowsDirectoryWatcher::new(&dir).unwrap();
    std::fs::write(dir.join("config.toml"), "a = 1").unwrap();
    assert_eq!(watcher.wait(LONG).unwrap(), DirectoryWake::Changed);

    // Act -- one save may signal more than once, but not for ever.
    let more = (0..10)
        .map(|_| watcher.wait(SHORT).unwrap())
        .take_while(|wake| *wake == DirectoryWake::Changed)
        .count();

    // Assert
    assert!(more < 10, "the watcher kept reporting the same change");
}

#[test]
fn a_directory_that_does_not_exist_cannot_be_watched() {
    // Arrange
    let dir = temp_dir("missing").join("not-there");

    // Act
    let result = WindowsDirectoryWatcher::new(&dir);

    // Assert
    assert!(
        matches!(
            result,
            Err(Error::Platform {
                operation: "watch the directory",
                ..
            })
        ),
        "{result:?}"
    );
}

#[test]
fn a_deleted_directory_ends_the_watch() {
    // Arrange
    let dir = temp_dir("deleted");
    let mut watcher = WindowsDirectoryWatcher::new(&dir).unwrap();

    // Act
    std::fs::remove_dir(&dir).unwrap();
    let end = (0..10)
        .map(|_| watcher.wait(LONG))
        .find(|wait| !matches!(wait, Ok(DirectoryWake::Changed)));

    // Assert -- the deletion is reported, then the handle cannot be
    // re-armed, so the watcher says the watch is over instead of waiting
    // for changes that will never come.
    assert!(
        matches!(
            end,
            Some(Err(Error::Platform {
                operation: "keep watching the directory",
                ..
            }))
        ),
        "{end:?}"
    );
}
