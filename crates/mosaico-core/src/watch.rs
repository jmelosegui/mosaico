//! Waiting for the files in a directory to change.

use std::time::Duration;

use crate::Result;

/// What ended a [`DirectoryWatcher::wait`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryWake {
    /// Something in the directory may have changed.
    Changed,
    /// The timeout passed with nothing to report.
    TimedOut,
}

/// Waits for the files in one directory to change.
///
/// It only says when to look: the config hot-reload sleeps on it, then
/// compares the files itself (see
/// [`ConfigWatch`](crate::config::ConfigWatch)).
///
/// # Contract for platform implementations
///
/// - A watcher is made for one directory that exists, and covers the
///   files directly in it: a file being created, deleted, renamed or
///   written to. Subdirectories and what they hold need not be covered.
///   Making a watcher for a directory that does not exist fails; it does
///   not wait for the directory to appear.
/// - No change is lost between waits: a change made since the watcher was
///   made, or since `wait` last returned [`DirectoryWake::Changed`], makes
///   the next `wait` return `Changed` at once.
/// - Spurious wake-ups are allowed: `Changed` may come for a file the
///   caller does not care about, more than once for one save, or for
///   nothing at all, so callers compare what they care about. A change
///   that was reported must not be reported again, or callers would spin.
/// - [`DirectoryWake::TimedOut`] comes once `timeout` has passed with
///   nothing to report, and never much later. A caller that has to be
///   able to stop waits in slices and checks in between.
/// - The watcher stays with the directory it was made for. When that
///   directory is deleted, `wait` fails once the platform notices, so the
///   caller can tell that the watch is over. A directory that is moved
///   may stay watched where it went. A directory created later at the
///   same path need not be watched.
/// - Dropping the watcher stops watching.
pub trait DirectoryWatcher {
    /// Blocks until something in the directory may have changed, or until
    /// `timeout` passes.
    ///
    /// # Errors
    ///
    /// Fails when the directory can no longer be watched, for instance
    /// because it was deleted. Nothing more is reported after that.
    fn wait(&mut self, timeout: Duration) -> Result<DirectoryWake>;
}

/// A lent watcher, so its owner can look at it once the watch is over.
impl<W: DirectoryWatcher + ?Sized> DirectoryWatcher for &mut W {
    fn wait(&mut self, timeout: Duration) -> Result<DirectoryWake> {
        (**self).wait(timeout)
    }
}
