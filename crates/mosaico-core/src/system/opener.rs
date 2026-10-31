//! Opening folders and files the way the user would.

use std::path::Path;

use crate::Result;

/// Opens a folder or file with what the user chose for it.
///
/// # Contract for platform implementations
///
/// - `open` shows a folder in the file manager, and opens a file in the
///   application the user chose for its type, and returns without
///   waiting for either.
/// - It fails with [`Error::Platform`](crate::Error::Platform) when
///   nothing is at `path` or nothing can open it.
pub trait Opener {
    /// Opens the folder or file at `path`.
    fn open(&self, path: &Path) -> Result<()>;
}
