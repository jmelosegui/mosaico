use std::path::PathBuf;

use super::*;
use crate::commands::test_support::{EXE, Harness, SETTINGS};

/// Where the settings app is installed in the fake world.
fn settings_exe() -> PathBuf {
    PathBuf::from(EXE).with_file_name(SETTINGS)
}

/// A harness for `test` with the settings app installed beside mosaico.
fn installed(test: &str) -> Harness {
    let h = Harness::new(test);
    h.world.files.borrow_mut().push(settings_exe());
    h
}

#[test]
fn opens_the_settings_app_installed_beside_mosaico() {
    // Arrange
    let mut h = installed("settings-opens");

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(*h.world.launched.borrow(), [settings_exe()]);
    assert_eq!(h.out(), "Opening settings...\n");
    assert_eq!(h.err(), "");
}

#[test]
fn a_missing_settings_app_is_reported_with_how_to_install_it() {
    // Arrange -- installed before the settings app shipped.
    let mut h = Harness::new("settings-missing");

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert!(h.world.launched.borrow().is_empty());
    assert_eq!(
        h.err(),
        format!(
            "Error: the settings app is not installed.\n\
             Expected it at {}.\n\
             Run 'mosaico update --force' to install it.\n",
            settings_exe().display()
        )
    );
}

#[test]
fn a_settings_app_that_cannot_start_is_reported() {
    // Arrange
    let mut h = installed("settings-fails");
    *h.world.launch_error.borrow_mut() = Some("access denied".into());

    // Act
    let exit = h.run(execute);

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.out(), "");
    assert_eq!(
        h.err(),
        "Error: could not start the program: access denied\n"
    );
}
