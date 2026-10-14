//! How a platform's release is packaged and installed.

use std::path::Path;

use crate::Result;

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
/// - `replace` puts an executable whose content is `binary` at `exe`.
///   The executable at `exe` may be running, since the caller usually
///   runs from it. After `Ok`, starting `exe` runs the new executable.
/// - When `replace` fails, the old executable is still at `exe` and
///   starts as before.
/// - `replace` may leave files beside `exe` that cannot be deleted while
///   the old executable runs. `remove_leftovers` deletes them, and
///   succeeds when there is nothing to delete. The daemon calls it when
///   it starts.
/// - A file that cannot be renamed, written or deleted is an
///   [`Error::Platform`](crate::Error::Platform) saying which.
pub trait Installer {
    /// The file name of this platform's release archive.
    fn asset_name(&self) -> &str;

    /// The file name of the mosaico executable in the archive.
    fn binary_name(&self) -> &str;

    /// Makes the executable at `exe` the one whose content is `binary`.
    fn replace(&self, exe: &Path, binary: &[u8]) -> Result<()>;

    /// Deletes what an earlier [`replace`](Self::replace) left beside
    /// `exe`.
    fn remove_leftovers(&self, exe: &Path) -> Result<()>;
}
