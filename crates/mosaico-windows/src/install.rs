//! [`Installer`] for the Windows release.
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

/// [`Installer`] that renames the running executable aside.
#[derive(Debug, Clone, Copy, Default)]
pub struct RenameInstaller;

impl Installer for RenameInstaller {
    /// The archive the release workflow builds for 64-bit Windows.
    fn asset_name(&self) -> &str {
        "mosaico-windows-amd64.zip"
    }

    fn binary_name(&self) -> &str {
        "mosaico.exe"
    }

    /// Renames `exe` to `.exe.old`, writes `binary` at `exe`, then tries
    /// to delete the old file. If the write fails, the old file is
    /// renamed back.
    fn replace(&self, exe: &Path, binary: &[u8]) -> Result<()> {
        let backup = backup_of(exe);

        // Remove leftover backup from a previous update.
        let _ = fs::remove_file(&backup);

        // Rename the running binary out of the way.
        fs::rename(exe, &backup)
            .map_err(|e| Error::platform("rename the running executable aside", e))?;

        // Write the new binary.
        if let Err(e) = fs::write(exe, binary) {
            // Attempt rollback.
            let _ = fs::rename(&backup, exe);
            return Err(Error::platform("write the new executable", e));
        }

        // Try to delete the backup (fails while it is still locked).
        let _ = fs::remove_file(&backup);
        Ok(())
    }

    /// Deletes the `.exe.old` beside `exe`, if there is one.
    fn remove_leftovers(&self, exe: &Path) -> Result<()> {
        match fs::remove_file(backup_of(exe)) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => {
                Err(Error::platform("delete the old executable", e))
            }
            _ => Ok(()),
        }
    }
}

/// Where `replace` moves the running executable: `mosaico.exe.old`.
fn backup_of(exe: &Path) -> PathBuf {
    exe.with_extension("exe.old")
}

#[cfg(test)]
#[path = "install_tests.rs"]
mod tests;
