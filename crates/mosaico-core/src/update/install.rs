//! How a platform's release is packaged and installed.

use std::path::{Path, PathBuf};

use crate::{Error, Result};

/// How mosaico is packaged for this platform, and how a downloaded
/// executable takes the place of the one that is running.
///
/// # Contract for platform implementations
///
/// - `asset_name` is the file name of this platform's archive among the
///   files attached to a mosaico release, built for the OS and CPU the
///   running executable was built for.
/// - `binary_name` is the file name of the mosaico executable inside
///   that archive.
/// - `settings_name` is the file name of the settings app's executable
///   inside that archive. It is installed in the same folder as the
///   mosaico executable, under the same name.
/// - `replace` puts an executable whose content is `binary` at `exe`.
///   The executable at `exe` may be running, since the caller usually
///   runs from it. After `Ok`, starting `exe` runs the new executable.
/// - When `replace` fails, the old executable is still at `exe` and
///   starts as before. If it cannot be put back there, the error says
///   where it is and how to restore it.
/// - `replace` may leave files beside `exe` that cannot be deleted while
///   the old executable runs. `remove_leftovers` deletes those left
///   beside the mosaico executable at `exe` and beside the settings app
///   in the same folder, and succeeds when there is nothing to delete.
///   The daemon calls it when it starts.
/// - A file that cannot be renamed, written or deleted is an
///   [`Error::Platform`](crate::Error::Platform) saying which.
/// - A platform that cannot update itself returns
///   [`Error::Unsupported`](crate::Error::Unsupported) from `asset_name`
///   and from `replace`, and its `remove_leftovers` succeeds. Callers ask
///   for `asset_name` before they stop anything or download.
///
/// # Installer packages
///
/// A platform may also publish mosaico as a package of its own installer,
/// which then owns the installed files: it upgrades and removes them, and
/// a package manager may track their version. Such an install must never
/// be updated with `replace`.
///
/// - `package_name` is the file name of that package among the files
///   attached to a release, for the OS and CPU of the running executable.
/// - `package_folder` is the folder the package installed mosaico into,
///   from what the package recorded when it was installed, or `None` when
///   no package is installed or that cannot be read.
/// - `run_package` hands `package`, the contents of a downloaded package,
///   to the system's installer and returns once the installer started,
///   without waiting for it. The installer must be able to replace the
///   executable the caller runs from after the caller exits. With
///   `reinstall`, it installs the package again over the same version.
/// - `set_aside_if_running` frees the file at `exe` for the installer
///   when a process runs from it, so the process keeps running while the
///   installer replaces or removes the file. It returns whether it moved
///   the file. A file no process runs from, or no file, is left alone.
///   What it leaves behind is deleted by `remove_leftovers`.
/// - A platform with no installer package keeps the default methods:
///   [`Error::Unsupported`](crate::Error::Unsupported) from
///   `package_name` and `run_package`, no `package_folder`, and nothing
///   set aside.
pub trait Installer {
    /// The file name of this platform's release archive.
    fn asset_name(&self) -> Result<&str>;

    /// The file name of the mosaico executable in the archive.
    fn binary_name(&self) -> &str;

    /// The file name of the settings app's executable, in the archive and
    /// beside the mosaico executable.
    fn settings_name(&self) -> &str;

    /// Makes the executable at `exe` the one whose content is `binary`,
    /// installing it when there is none there yet.
    fn replace(&self, exe: &Path, binary: &[u8]) -> Result<()>;

    /// Deletes what earlier [`replace`](Self::replace)s left beside the
    /// mosaico executable at `exe` and the settings app next to it.
    fn remove_leftovers(&self, exe: &Path) -> Result<()>;

    /// The file name of this platform's installer package.
    fn package_name(&self) -> Result<&str> {
        Err(Error::Unsupported("installer package"))
    }

    /// The folder this platform's installer package installed mosaico
    /// into, if it did.
    fn package_folder(&self) -> Option<PathBuf> {
        None
    }

    /// Starts the system's installer on `package` without waiting for it.
    fn run_package(&self, _package: &[u8], _reinstall: bool) -> Result<()> {
        Err(Error::Unsupported("installer package"))
    }

    /// Moves the file at `exe` out of an installer's way if a process
    /// runs from it, and says whether it did.
    fn set_aside_if_running(&self, _exe: &Path) -> Result<bool> {
        Ok(false)
    }
}
