//! The MSI package of the Windows release, for the installer package
//! methods of [`RenameInstaller`](super::RenameInstaller).
//!
//! The package (`packaging/windows/mosaico.wxs`) installs per user, in
//! `%LOCALAPPDATA%\Programs\mosaico`, and records that folder as the
//! `InstallFolder` value under `HKEY_CURRENT_USER\Software\mosaico`. The
//! value is removed with the package, so it names the folder of the
//! package that is installed now.
//!
//! A downloaded package is run with `msiexec /i ... /passive`, which
//! shows a progress bar and asks nothing. msiexec only replaces files
//! once its own checks are done, which takes longer than the `mosaico
//! update` that started it needs to exit, so the file that process runs
//! from is free by then.

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use mosaico_core::{Error, Result};

use crate::autostart::registry::Value;
use crate::error::OrPlatform;
use crate::process::spawn_detached;

use super::backup_of;

/// Where the package records the folder it installed mosaico into.
const INSTALL_FOLDER: Value = Value {
    key: r"Software\mosaico",
    name: "InstallFolder",
};

/// What Windows answers when a file is opened for writing while a process
/// runs from it: `ERROR_SHARING_VIOLATION`.
const SHARING_VIOLATION: i32 = 32;

/// The release's MSI package for `arch`, Rust's name for the CPU of a
/// build, as [`archive_for`](super::archive_for) picks the archive.
pub(super) fn package_for(arch: &str) -> Result<&'static str> {
    match arch {
        "x86_64" => Ok("mosaico-windows-amd64.msi"),
        "aarch64" => Ok("mosaico-windows-arm64.msi"),
        _ => Err(Error::Unsupported("installer package")),
    }
}

/// The folder the package recorded, if it is installed. A value that
/// cannot be read counts as no package, so mosaico updates itself as it
/// would without one.
pub(super) fn package_folder() -> Option<PathBuf> {
    INSTALL_FOLDER
        .read_string("read the installer's folder")
        .ok()
        .flatten()
        .filter(|folder| !folder.is_empty())
        .map(PathBuf::from)
}

/// Writes `package` to `name` in a folder of the temporary directory and
/// starts msiexec on it, logging beside it, without waiting.
pub(super) fn run_package(name: &str, package: &[u8], reinstall: bool) -> Result<()> {
    let dir = std::env::temp_dir().join("mosaico-update");
    fs::create_dir_all(&dir).or_platform("create a folder for the installer")?;
    let path = dir.join(name);
    fs::write(&path, package).or_platform("save the installer")?;

    let log = dir.join("install.log");
    let mut args = vec![
        OsStr::new("/i"),
        path.as_os_str(),
        OsStr::new("/passive"),
        OsStr::new("/norestart"),
        OsStr::new("/l*v"),
        log.as_os_str(),
    ];
    if reinstall {
        // The same version is installed already: reinstall every file
        // from this package instead of only repairing what is missing.
        args.extend(["REINSTALL=ALL", "REINSTALLMODE=vamus"].map(OsStr::new));
    }
    spawn_detached(OsStr::new("msiexec"), &args, "start the installer")?;
    Ok(())
}

/// Renames `exe` to `.exe.old` when a process runs from it, which Windows
/// allows even though it does not allow writing to it.
pub(super) fn set_aside_if_running(exe: &Path) -> Result<bool> {
    match fs::OpenOptions::new().write(true).open(exe) {
        Ok(_) => Ok(false),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) if e.raw_os_error() == Some(SHARING_VIOLATION) => {
            let backup = backup_of(exe);
            let _ = fs::remove_file(&backup);
            fs::rename(exe, &backup).or_platform("move the running executable aside")?;
            Ok(true)
        }
        Err(e) => Err(e).or_platform("check whether the executable runs"),
    }
}

#[cfg(test)]
#[path = "install_package_tests.rs"]
mod tests;
