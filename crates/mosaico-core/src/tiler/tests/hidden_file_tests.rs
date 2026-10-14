use std::path::{Path, PathBuf};

use crate::WindowId;
use crate::config::HidingBehaviour;
use crate::tiler::{Effect, WindowIdentity};

use super::*;

/// A directory of a test's own in the temp directory, removed when the
/// test ends.
struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
        // The parent is shared with the other tests, which run at the
        // same time, so it is left in place.
    }
}

/// A record in a directory of the test's own, with nothing in it yet.
fn temp_file(test: &str) -> (HiddenFile, TempDir) {
    let dir = std::env::temp_dir()
        .join("mosaico-hidden-file-tests")
        .join(format!("{test}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    (HiddenFile::new(dir.join(FILE_NAME)), TempDir(dir))
}

fn one_hidden() -> HiddenWindows {
    let mut record = HiddenWindows::default();
    record.applied(
        &Effect::Hide {
            window: WindowId::from_raw(0x4014E),
            how: HidingBehaviour::Cloak,
        },
        |_| {
            Some(WindowIdentity {
                process: 7,
                class: "App".into(),
            })
        },
    );
    record
}

#[test]
fn the_record_of_an_instance_is_in_its_data_directory() {
    // Arrange
    let instance = Instance::new(Some("work"), "config", "data").unwrap();

    // Act
    let file = HiddenFile::for_instance(&instance);

    // Assert
    assert_eq!(file.path, Path::new("data").join(FILE_NAME));
}

#[test]
fn a_missing_file_is_an_empty_record() {
    // Arrange
    let (file, _dir) = temp_file("missing");

    // Act / Assert
    assert!(file.load().is_empty());
}

#[test]
fn a_saved_record_loads_back() {
    // Arrange
    let (file, _dir) = temp_file("round-trip");
    let record = one_hidden();

    // Act
    file.save(&record);

    // Assert
    assert_eq!(file.load(), record);
}

#[test]
fn saving_an_empty_record_removes_the_file() {
    // Arrange
    let (file, _dir) = temp_file("empty");
    file.save(&one_hidden());

    // Act
    file.save(&HiddenWindows::default());

    // Assert
    assert!(!file.path.exists());
}

#[test]
fn a_corrupt_file_is_an_empty_record() {
    // Arrange
    let (file, _dir) = temp_file("corrupt");
    fs::create_dir_all(file.path.parent().unwrap()).unwrap();
    fs::write(&file.path, "{ this is not a record").unwrap();

    // Act / Assert
    assert!(file.load().is_empty());
}
