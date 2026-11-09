use std::path::PathBuf;

use mosaico_core::ipc::Command;

use crate::commands::context::Exit;
use crate::commands::test_support::{EXE, Harness, PACKAGE};
use crate::commands::update::execute;

/// The version the tests run as.
const LOCAL: &str = env!("CARGO_PKG_VERSION");

/// A harness for `test` in which mosaico runs from the folder the
/// installer package installed it into and the latest release is `tag`,
/// with its package attached.
fn packaged(test: &str, tag: &str) -> Harness {
    let h = Harness::new(test);
    let folder = PathBuf::from(EXE).parent().unwrap().to_path_buf();
    *h.world.installer.package_folder.borrow_mut() = Some(folder);
    h.world.releases.publish(tag);
    h.world.releases.attach(PACKAGE, b"new package".to_vec());
    h
}

#[test]
fn a_packaged_mosaico_is_updated_by_running_the_new_package() {
    // Arrange
    let mut h = packaged("update-package", "v99.0.0");
    h.world.daemon_up(4321);

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert -- the package's hooks stop and restart the daemon.
    assert_eq!(exit, Exit::Success, "{}", h.err());
    assert_eq!(
        *h.world.installer.packages_run.borrow(),
        [(b"new package".to_vec(), false)]
    );
    assert_eq!(*h.world.installer.replaced.borrow(), []);
    assert_eq!(*h.world.sent.borrow(), Vec::<Command>::new());
    assert_eq!(
        h.out(),
        format!(
            "Checking for updates... v{LOCAL} -> v99.0.0.\n\
             Downloading the installer for v99.0.0...\n\
             The installer is running: it stops mosaico, installs v99.0.0 and starts \
             mosaico again if it was running.\n"
        )
    );
}

#[test]
fn forcing_the_installed_version_reinstalls_its_package() {
    // Arrange
    let mut h = packaged("update-package-force", &format!("v{LOCAL}"));

    // Act
    let exit = h.run(|cx| execute(cx, true));

    // Assert
    assert_eq!(exit, Exit::Success, "{}", h.err());
    assert_eq!(
        *h.world.installer.packages_run.borrow(),
        [(b"new package".to_vec(), true)]
    );
}

#[test]
fn a_release_without_a_package_fails_without_touching_mosaico() {
    // Arrange
    let mut h = Harness::new("update-package-missing");
    let folder = PathBuf::from(EXE).parent().unwrap().to_path_buf();
    *h.world.installer.package_folder.borrow_mut() = Some(folder);
    h.world.releases.publish("v99.0.0");
    h.world.daemon_up(4321);

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert!(h.err().contains("404"), "{}", h.err());
    assert_eq!(*h.world.installer.packages_run.borrow(), []);
    assert!(h.world.daemon_running.get());
}

#[test]
fn a_package_that_cannot_start_fails_the_update() {
    // Arrange
    let mut h = packaged("update-package-start", "v99.0.0");
    *h.world.installer.package_error.borrow_mut() = Some("denied".into());

    // Act
    let exit = h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(exit, Exit::Failure);
    assert!(h.err().contains("denied"), "{}", h.err());
}

#[test]
fn a_copy_outside_the_package_folder_updates_itself() {
    // Arrange
    let mut h = packaged("update-package-copy", "v99.0.0");
    *h.world.installer.package_folder.borrow_mut() = Some(PathBuf::from(r"C:\elsewhere"));

    // Act -- the release has no archive, so replacing fails, but only
    // after it was tried.
    h.run(|cx| execute(cx, false));

    // Assert
    assert_eq!(*h.world.installer.packages_run.borrow(), []);
    assert!(h.out().contains("Downloading v99.0.0...\n"), "{}", h.out());
}
