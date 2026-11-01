use std::io::{Cursor, Write};
use std::path::PathBuf;

use mosaico_core::ipc::Command;
use zip::write::SimpleFileOptions;

use super::*;
use crate::commands::test_support::{ASSET, BINARY, EXE, Harness, SETTINGS};

/// The version the tests run as.
const LOCAL: &str = env!("CARGO_PKG_VERSION");

/// A zip archive holding `files`, as names and contents.
fn zip_of(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, content) in files {
        zip.start_file(*name, SimpleFileOptions::default()).unwrap();
        zip.write_all(content).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

/// A harness for `test` in which the latest release is `tag`, with this
/// platform's archive holding the binary `new binary` and the settings
/// app `new settings`.
fn released(test: &str, tag: &str) -> Harness {
    let h = Harness::new(test);
    h.world.releases.publish(tag);
    let archive = zip_of(&[
        ("README.md", b"read me"),
        (BINARY, b"new binary"),
        (SETTINGS, b"new settings"),
    ]);
    h.world.releases.attach(ASSET, archive);
    h
}

#[test]
fn says_so_when_already_on_the_latest_version() {
    // Arrange
    let mut h = released("update-latest", &format!("v{LOCAL}"));

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(
        h.out(),
        format!("Checking for updates...\nYou are already on the latest version (v{LOCAL}).\n")
    );
    assert_eq!(h.err(), "");
    assert_eq!(*h.world.installer.replaced.borrow(), []);
}

#[test]
fn fails_when_github_cannot_be_reached() {
    // Arrange -- nothing is published, so GitHub cannot be reached.
    let mut h = Harness::new("update-offline");

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(h.out(), "Checking for updates... failed.\n");
    assert_eq!(
        h.err(),
        "Error: could not reach GitHub to check for updates.\n"
    );
}

#[test]
fn installs_a_newer_release_over_the_running_executable() {
    // Arrange
    let mut h = released("update-newer", "v99.0.0");

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert -- the settings app first, so a failure keeps the old mosaico.
    assert_eq!(exit, Exit::Success);
    assert_eq!(
        h.out(),
        format!(
            "Checking for updates... v{LOCAL} -> v99.0.0.\n\
             Downloading v99.0.0...\n\
             Installing to {EXE}...\n\
             Updated successfully: v{LOCAL} -> v99.0.0.\n"
        )
    );
    assert_eq!(h.err(), "");
    let settings = PathBuf::from(EXE).with_file_name(SETTINGS);
    assert_eq!(
        *h.world.installer.replaced.borrow(),
        [
            (settings, b"new settings".to_vec()),
            (PathBuf::from(EXE), b"new binary".to_vec()),
        ]
    );
}

#[test]
fn a_failed_settings_install_leaves_the_executable_alone() {
    // Arrange
    let mut h = released("update-settings-fails", "v99.0.0");
    *h.world.installer.error.borrow_mut() = Some(("in use".into(), Some(SETTINGS)));

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(
        h.err(),
        "Error: could not write the new executable: in use\n"
    );
    assert_eq!(*h.world.installer.replaced.borrow(), []);
}

#[test]
fn an_archive_without_the_settings_app_leaves_the_executable_alone() {
    // Arrange -- as archives were before the settings app shipped.
    let mut h = Harness::new("update-no-settings");
    h.world.releases.publish("v99.0.0");
    h.world
        .releases
        .attach(ASSET, zip_of(&[(BINARY, b"new binary")]));

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(
        h.err(),
        format!("Error: {SETTINGS} not found in zip archive\n")
    );
    assert_eq!(*h.world.installer.replaced.borrow(), []);
}

#[test]
fn reinstalls_the_running_version_when_forced() {
    // Arrange
    let mut h = released("update-forced", &format!("v{LOCAL}"));

    // Act
    let exit = h.run(|cx| execute(cx, true));

    // Assert
    assert_eq!(exit, Exit::Success);
    let out = h.out();
    assert!(
        out.starts_with(&format!("Checking for updates... reinstalling v{LOCAL}.\n")),
        "{out}"
    );
    assert!(out.ends_with(&format!("Reinstalled v{LOCAL}.\n")), "{out}");
    assert_eq!(h.world.installer.replaced.borrow().len(), 2);
}

#[test]
fn stops_a_running_daemon_to_install_and_starts_it_again() {
    // Arrange
    let mut h = released("update-daemon", "v99.0.0");
    h.world.daemon_up(10);

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(*h.world.sent.borrow(), [Command::Stop]);
    assert!(h.world.daemon_running.get(), "started again");
    let out = h.out();
    let stopping = out.find("Stopping daemon...").unwrap();
    let installing = out.find("Installing to").unwrap();
    let restarting = out.find("Restarting daemon...").unwrap();
    assert!(stopping < installing && installing < restarting, "{out}");
}

#[test]
fn the_restarted_daemon_is_announced_with_the_installed_version() {
    // Arrange -- updating from an older version, which restarts the
    // daemon from the process that is still the old version.
    let mut h = released("update-banner", "v99.0.0");
    h.world.daemon_up(10);

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(exit, Exit::Success);
    assert_eq!(*h.world.logos.borrow(), ["99.0.0"]);
    let out = h.out();
    assert!(!out.contains("Update available"), "{out}");
}

#[test]
fn a_release_without_this_platforms_archive_leaves_the_executable_alone() {
    // Arrange -- the release is out, but nothing is attached to it yet.
    let mut h = Harness::new("update-no-asset");
    h.world.releases.publish("v99.0.0");
    h.world.daemon_up(10);

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(
        h.err(),
        format!(
            "Error: network request failed: \
             https://github.com/jmelosegui/mosaico/releases/download/v99.0.0/{ASSET} \
             answered with HTTP status 404\n\
             The daemon was stopped before the update failed.\n\
             Run 'mosaico start' to restart with the current version.\n"
        )
    );
    assert_eq!(*h.world.installer.replaced.borrow(), []);
    assert!(!h.world.daemon_running.get(), "not started again");
}

#[test]
fn an_archive_without_the_binary_leaves_the_executable_alone() {
    // Arrange
    let mut h = Harness::new("update-no-binary");
    h.world.releases.publish("v99.0.0");
    h.world
        .releases
        .attach(ASSET, zip_of(&[("README.md", b"read me")]));

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(
        h.err(),
        format!("Error: {BINARY} not found in zip archive\n")
    );
    assert_eq!(*h.world.installer.replaced.borrow(), []);
}

#[test]
fn a_download_that_is_not_an_archive_leaves_the_executable_alone() {
    // Arrange
    let mut h = Harness::new("update-not-zip");
    h.world.releases.publish("v99.0.0");
    h.world.releases.attach(ASSET, b"Not Found".to_vec());

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert!(
        h.err().starts_with("Error: invalid zip archive: "),
        "{}",
        h.err()
    );
    assert_eq!(*h.world.installer.replaced.borrow(), []);
}

#[test]
fn a_failed_install_is_reported_with_how_to_get_the_daemon_back() {
    // Arrange
    let mut h = released("update-install-fails", "v99.0.0");
    h.world.daemon_up(10);
    *h.world.installer.error.borrow_mut() = Some(("access denied".into(), None));

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(
        h.err(),
        "Error: could not write the new executable: access denied\n\
         The daemon was stopped before the update failed.\n\
         Run 'mosaico start' to restart with the current version.\n"
    );
    assert!(!h.world.daemon_running.get(), "not started again");
}

#[test]
fn says_so_before_stopping_anything_on_a_platform_that_cannot_update_itself() {
    // Arrange
    let mut h = released("update-unsupported", "v99.0.0");
    h.world.daemon_up(10);
    h.world.installer.unsupported.set(true);

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert_eq!(
        h.out(),
        format!("Checking for updates... v{LOCAL} -> v99.0.0.\n")
    );
    assert_eq!(
        h.err(),
        "Error: self-update is not supported on this platform\n"
    );
    assert!(
        h.world.sent.borrow().is_empty(),
        "the daemon was not asked to stop"
    );
    assert!(h.world.daemon_running.get());
}
