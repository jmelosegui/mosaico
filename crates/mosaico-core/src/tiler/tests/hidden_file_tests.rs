use std::path::Path;

use crate::WindowId;
use crate::config::HidingBehaviour;
use crate::tiler::{Effect, WindowIdentity};

use super::*;

fn temp_file(test: &str) -> HiddenFile {
    let dir = std::env::temp_dir()
        .join("mosaico-hidden-file-tests")
        .join(format!("{test}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    HiddenFile::new(dir.join(FILE_NAME))
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
    let file = temp_file("missing");

    // Act / Assert
    assert!(file.load().is_empty());
}

#[test]
fn a_saved_record_loads_back() {
    // Arrange
    let file = temp_file("round-trip");
    let record = one_hidden();

    // Act
    file.save(&record);

    // Assert
    assert_eq!(file.load(), record);
}

#[test]
fn saving_an_empty_record_removes_the_file() {
    // Arrange
    let file = temp_file("empty");
    file.save(&one_hidden());

    // Act
    file.save(&HiddenWindows::default());

    // Assert
    assert!(!file.path.exists());
}

#[test]
fn a_corrupt_file_is_an_empty_record() {
    // Arrange
    let file = temp_file("corrupt");
    fs::create_dir_all(file.path.parent().unwrap()).unwrap();
    fs::write(&file.path, "{ this is not a record").unwrap();

    // Act / Assert
    assert!(file.load().is_empty());
}
