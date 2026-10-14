//! [`Installer`](mosaico_core::update::Installer) for the Windows
//! release.
//!
//! Windows lets a running executable be renamed but not overwritten, so
//! `replace` renames `mosaico.exe` to `mosaico.exe.old` and writes the
//! new binary in its place. The old file stays locked while a process
//! still runs from it, so deleting it can fail right after the update;
//! the next daemon start deletes it through `remove_leftovers`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use mosaico_core::update::Installer;
use mosaico_core::{Error, Result};

use crate::error::OrPlatform;

/// [`Installer`] that renames the running executable aside.
#[derive(Debug, Clone, Copy, Default)]
pub struct RenameInstaller;

impl Installer for RenameInstaller {
    /// The archive for the CPU this binary was built for:
    /// `mosaico-windows-amd64.zip` for x86-64, `mosaico-windows-arm64.zip`
    /// for ARM64. A build for another CPU cannot update itself.
    fn asset_name(&self) -> Result<&str> {
        archive_for(std::env::consts::ARCH)
    }

    fn binary_name(&self) -> &str {
        "mosaico.exe"
    }

    /// Renames `exe` to `.exe.old`, writes `binary` at `exe`, then tries
    /// to delete the old file. If the write fails, the old file is
    /// renamed back.
    fn replace(&self, exe: &Path, binary: &[u8]) -> Result<()> {
        replace(&Disk, exe, binary)
    }

    /// Deletes the `.exe.old` beside `exe`, if there is one.
    fn remove_leftovers(&self, exe: &Path) -> Result<()> {
        match fs::remove_file(backup_of(exe)) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            removed => removed.or_platform("delete the old executable"),
        }
    }
}

/// The file operations `replace` is made of, so tests can make one of
/// them fail.
trait Files {
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn write(&self, path: &Path, contents: &[u8]) -> io::Result<()>;
    fn remove(&self, path: &Path) -> io::Result<()>;
}

/// The real file system.
struct Disk;

impl Files for Disk {
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn write(&self, path: &Path, contents: &[u8]) -> io::Result<()> {
        fs::write(path, contents)
    }

    fn remove(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }
}

/// [`RenameInstaller::replace`] on `files`.
fn replace(files: &impl Files, exe: &Path, binary: &[u8]) -> Result<()> {
    let backup = backup_of(exe);

    // Remove leftover backup from a previous update.
    let _ = files.remove(&backup);

    // Rename the running binary out of the way.
    files
        .rename(exe, &backup)
        .or_platform("rename the running executable aside")?;

    // Write the new binary.
    if let Err(e) = files.write(exe, binary) {
        // Put the old binary back. When that fails too, `exe` is gone
        // or holds part of the new one, so the error says where the
        // working one is.
        if let Err(back) = files.rename(&backup, exe) {
            let name = exe.file_name().unwrap_or_default().to_string_lossy();
            let where_it_is = format!(
                "{back} (writing the new one had failed: {e}). It is at {}: \
                 rename it to {name} to restore it",
                backup.display()
            );
            return Err(Error::platform("put the old executable back", where_it_is));
        }
        return Err(e).or_platform("write the new executable");
    }

    // Try to delete the backup (fails while it is still locked).
    let _ = files.remove(&backup);
    Ok(())
}

/// The release archive for `arch`, Rust's name for the CPU of a build.
///
/// Only the x86-64 archive is published so far. An ARM64 build asks for
/// the ARM64 one, so updating it fails on the missing archive instead of
/// installing an x86-64 binary over it. A build for any other CPU has no
/// archive, so it cannot update itself.
fn archive_for(arch: &str) -> Result<&'static str> {
    match arch {
        "x86_64" => Ok("mosaico-windows-amd64.zip"),
        "aarch64" => Ok("mosaico-windows-arm64.zip"),
        _ => Err(Error::Unsupported("self-update")),
    }
}

/// Where `replace` moves the running executable: `mosaico.exe.old`.
fn backup_of(exe: &Path) -> PathBuf {
    exe.with_extension("exe.old")
}

#[cfg(test)]
#[path = "install_tests.rs"]
mod tests;
