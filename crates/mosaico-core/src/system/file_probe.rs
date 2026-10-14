//! Asking the file system about a program without waiting on it.

use std::path::Path;

/// Quick questions about program files, for telling whether what is
/// registered to start at login is the running mosaico.
///
/// # Contract for platform implementations
///
/// - Neither method may wait on the network: a path on a network share,
///   or on a drive that maps one, is answered without looking at it.
/// - `is_file` is `Some(true)` when a file is at `path`, `Some(false)`
///   when nothing is there, or something that is not a file such as a
///   directory, and `None` when it cannot tell without waiting.
/// - `same_file` is whether `a` and `b` name the same existing file,
///   however each is spelled: letter case where the file system ignores
///   it, other names the system keeps for the same path, and a symbolic
///   link or junction to the file. A hard link counts as another file,
///   and so does a path that cannot be checked without waiting.
pub trait FileProbe {
    /// Whether a file is at `path`, or `None` when that cannot be told
    /// without waiting.
    fn is_file(&self, path: &Path) -> Option<bool>;

    /// Whether `a` and `b` name the same file.
    fn same_file(&self, a: &Path, b: &Path) -> bool;
}
