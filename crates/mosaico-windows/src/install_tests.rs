use std::fs;
use std::path::PathBuf;

use mosaico_core::Error;

use super::*;

/// A `mosaico.exe` holding `old` in a fresh directory for `test`.
fn installed(test: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("mosaico-install-tests")
        .join(format!("{test}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("mosaico.exe");
    fs::write(&exe, "old").unwrap();
    exe
}

/// The disk, except for the steps told to fail.
#[derive(Default)]
struct Flaky {
    /// Writing stops halfway and fails, as on a full disk.
    write_fails: bool,
    /// Renaming onto this path fails, as while it is locked.
    rename_onto_fails: Option<PathBuf>,
}

impl Files for Flaky {
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        if self.rename_onto_fails.as_deref() == Some(to) {
            return Err(io::Error::other("the file is locked"));
        }
        Disk.rename(from, to)
    }

    fn write(&self, path: &Path, contents: &[u8]) -> io::Result<()> {
        if self.write_fails {
            Disk.write(path, &contents[..contents.len() / 2])?;
            return Err(io::Error::other("the disk is full"));
        }
        Disk.write(path, contents)
    }

    fn remove(&self, path: &Path) -> io::Result<()> {
        Disk.remove(path)
    }
}

#[test]
fn a_failed_write_puts_the_old_executable_back() {
    // Arrange
    let exe = installed("write-fails");
    let files = Flaky {
        write_fails: true,
        ..Flaky::default()
    };

    // Act
    let error = replace(&files, &exe, b"new binary").unwrap_err();

    // Assert
    assert!(
        matches!(
            error,
            Error::Platform {
                operation: "write the new executable",
                ..
            }
        ),
        "{error:?}"
    );
    assert_eq!(fs::read(&exe).unwrap(), b"old");
    assert!(!exe.with_extension("exe.old").exists());
}

#[test]
fn each_64_bit_cpu_has_the_archive_built_for_it() {
    // Act / Assert
    assert_eq!(archive_for("x86_64").unwrap(), "mosaico-windows-amd64.zip");
    assert_eq!(archive_for("aarch64").unwrap(), "mosaico-windows-arm64.zip");
}

#[test]
fn a_build_for_another_cpu_cannot_update_itself() {
    // Act
    let error = archive_for("x86").unwrap_err();

    // Assert
    assert!(matches!(error, Error::Unsupported(_)), "{error:?}");
}

#[test]
fn this_build_asks_for_the_archive_of_its_own_cpu_with_mosaico_exe() {
    // Act / Assert
    assert_eq!(
        RenameInstaller.asset_name().unwrap(),
        archive_for(std::env::consts::ARCH).unwrap()
    );
    assert_eq!(RenameInstaller.binary_name(), "mosaico.exe");
    assert_eq!(RenameInstaller.settings_name(), "mosaico-settings.exe");
}

#[test]
fn replacing_puts_the_new_binary_in_place_and_leaves_no_backup() {
    // Arrange
    let exe = installed("replace");

    // Act
    RenameInstaller.replace(&exe, b"new").unwrap();

    // Assert
    assert_eq!(fs::read(&exe).unwrap(), b"new");
    assert!(!exe.with_extension("exe.old").exists());
}

#[test]
fn replacing_first_removes_a_backup_left_by_an_earlier_update() {
    // Arrange
    let exe = installed("stale-backup");
    fs::write(exe.with_extension("exe.old"), "older").unwrap();

    // Act
    RenameInstaller.replace(&exe, b"new").unwrap();

    // Assert
    assert_eq!(fs::read(&exe).unwrap(), b"new");
    assert!(!exe.with_extension("exe.old").exists());
}

#[test]
fn replacing_installs_an_executable_that_is_not_there_yet() {
    // Arrange -- an install from before the settings app shipped has no
    // mosaico-settings.exe for the update to rename aside.
    let exe = installed("missing").with_file_name("mosaico-settings.exe");

    // Act
    let result = RenameInstaller.replace(&exe, b"new");

    // Assert
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(fs::read(&exe).unwrap(), b"new");
    assert!(!exe.with_extension("exe.old").exists());
}

#[test]
fn the_backup_of_an_earlier_update_is_removed_as_a_leftover() {
    // Arrange
    let exe = installed("leftovers");
    fs::write(exe.with_extension("exe.old"), "older").unwrap();

    // Act
    RenameInstaller.remove_leftovers(&exe).unwrap();

    // Assert
    assert!(!exe.with_extension("exe.old").exists());
    assert_eq!(fs::read(&exe).unwrap(), b"old", "the executable stays");
}

#[test]
fn the_backup_of_the_settings_app_is_removed_as_a_leftover() {
    // Arrange
    let exe = installed("settings-leftovers");
    let backup = exe.with_file_name("mosaico-settings.exe.old");
    fs::write(&backup, "older").unwrap();

    // Act
    RenameInstaller.remove_leftovers(&exe).unwrap();

    // Assert
    assert!(!backup.exists());
}

#[test]
fn removing_leftovers_when_there_are_none_succeeds() {
    // Arrange
    let exe = installed("no-leftovers");

    // Act
    let result = RenameInstaller.remove_leftovers(&exe);

    // Assert
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn an_executable_that_cannot_be_put_back_is_named_where_it_is() {
    // Arrange -- the write fails and leaves a part of the new binary
    // that cannot be replaced, so the old one cannot go back.
    let exe = installed("rollback-fails");
    let files = Flaky {
        write_fails: true,
        rename_onto_fails: Some(exe.clone()),
    };

    // Act
    let error = replace(&files, &exe, b"new binary").unwrap_err();

    // Assert
    assert!(
        matches!(
            error,
            Error::Platform {
                operation: "put the old executable back",
                ..
            }
        ),
        "{error:?}"
    );
    let backup = exe.with_extension("exe.old");
    let text = error.to_string();
    assert!(text.contains(&backup.display().to_string()), "{text}");
    assert!(text.contains("rename it to mosaico.exe"), "{text}");
    assert_eq!(fs::read(&backup).unwrap(), b"old", "the old one is kept");
}
