use std::cell::Cell;

use mosaico_core::system::LaunchCommand;

use super::*;
use crate::commands::test_support::{EXE, Harness};

/// What [`offers_autostart`] says in `h`, given whether `init` wrote
/// `config.toml`.
fn offers(h: &mut Harness, created_config: bool) -> bool {
    let offered = Cell::new(false);
    h.run(|cx| {
        offered.set(offers_autostart(cx, created_config));
        Exit::Success
    });
    offered.get()
}

#[test]
fn every_file_is_written_into_a_config_folder_that_does_not_exist_yet() {
    // Arrange
    let mut h = Harness::new("init-fresh");
    let store = h.config.clone();
    let written = Cell::new(None);

    // Act
    h.run(|cx| {
        written.set(write_files(cx, &store));
        Exit::Success
    });

    // Assert
    assert_eq!(written.get(), Some(true));
    let created = h
        .out()
        .lines()
        .filter(|l| l.starts_with("Created "))
        .count();
    assert_eq!(created, 5, "{}", h.out());
    assert!(store.bar_path().is_file());
}

#[test]
fn a_config_the_daemon_wrote_and_nobody_changed_still_offers_autostart() {
    // Arrange -- a fresh install whose daemon started before `init`.
    let mut h = Harness::new("init-daemon-first");
    h.config.write_missing_defaults().unwrap();

    // Act
    let offered = offers(&mut h, false);

    // Assert
    assert!(offered);
}

#[test]
fn an_edited_config_or_a_registered_autostart_is_not_a_first_run() {
    // Arrange
    let mut edited = Harness::new("init-edited");
    edited.config.write_missing_defaults().unwrap();
    std::fs::write(edited.config.config_path(), "[layout]\ngap = 3\n").unwrap();
    let mut registered = Harness::new("init-registered");
    registered.config.write_missing_defaults().unwrap();
    *registered.world.autostart.borrow_mut() = Some(LaunchCommand::autostart(EXE));

    // Act
    let edited = offers(&mut edited, false);
    let registered = offers(&mut registered, false);

    // Assert
    assert!(!edited);
    assert!(!registered);
}
