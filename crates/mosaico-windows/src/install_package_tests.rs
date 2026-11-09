use std::fs::{self, File};
use std::os::windows::fs::OpenOptionsExt;
use std::path::PathBuf;

use mosaico_core::Error;

use super::*;

/// Who else may open a file, as `FILE_SHARE_READ | FILE_SHARE_DELETE`:
/// what Windows lets others do with the executable of a running process.
const SHARED_AS_A_RUNNING_EXE: u32 = 0x1 | 0x4;

/// A `mosaico-settings.exe` in a fresh directory for `test`.
fn installed(test: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("mosaico-install-package-tests")
        .join(format!("{test}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("mosaico-settings.exe");
    fs::write(&exe, "settings").unwrap();
    exe
}

/// Holds `exe` open the way Windows holds the file of a running process.
fn running(exe: &Path) -> File {
    fs::OpenOptions::new()
        .read(true)
        .share_mode(SHARED_AS_A_RUNNING_EXE)
        .open(exe)
        .unwrap()
}

#[test]
fn each_cpu_has_its_own_package() {
    // Act / Assert
    assert_eq!(package_for("x86_64").unwrap(), "mosaico-windows-amd64.msi");
    assert_eq!(package_for("aarch64").unwrap(), "mosaico-windows-arm64.msi");
    assert!(matches!(package_for("x86"), Err(Error::Unsupported(_))));
}

#[test]
fn an_executable_nothing_runs_from_stays() {
    // Arrange
    let exe = installed("idle");

    // Act
    let moved = set_aside_if_running(&exe).unwrap();

    // Assert
    assert!(!moved);
    assert_eq!(fs::read_to_string(&exe).unwrap(), "settings");
}

#[test]
fn a_missing_executable_is_not_running() {
    // Arrange
    let exe = installed("missing");
    fs::remove_file(&exe).unwrap();

    // Act
    let moved = set_aside_if_running(&exe).unwrap();

    // Assert
    assert!(!moved);
}

#[test]
fn a_running_executable_is_renamed_aside() {
    // Arrange
    let exe = installed("running");
    let backup = backup_of(&exe);
    fs::write(&backup, "older").unwrap();
    let process = running(&exe);

    // Act
    let moved = set_aside_if_running(&exe).unwrap();

    // Assert
    drop(process);
    assert!(moved);
    assert!(!exe.exists());
    assert_eq!(fs::read_to_string(&backup).unwrap(), "settings");
}
