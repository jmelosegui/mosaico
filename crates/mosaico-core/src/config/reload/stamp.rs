//! What a watched file looked like when last seen.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;
use std::time::SystemTime;

/// A watched file as last seen: when it was modified and what it held.
///
/// Both count, because a save can truncate a file and write it again with
/// one modification time. Seen in between, the time alone would mark the
/// save as seen while the file held only part of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Stamp {
    /// When the file was last modified, or `None` when that cannot be
    /// read, as for a file that does not exist.
    modified: Option<SystemTime>,
    /// A hash of what the file holds and its length, or `None` when it
    /// cannot be read.
    content: Option<(u64, usize)>,
}

impl Stamp {
    /// The file at `path` as it is now.
    pub(super) fn of(path: &Path) -> Self {
        let modified = path.metadata().ok().and_then(|m| m.modified().ok());
        let content = std::fs::read(path).ok().map(|bytes| {
            let mut hasher = DefaultHasher::new();
            bytes.hash(&mut hasher);
            (hasher.finish(), bytes.len())
        });
        Self { modified, content }
    }

    /// Whether the file exists and holds nothing, as it does between the
    /// two steps of a save that truncates it.
    pub(super) fn is_empty(&self) -> bool {
        matches!(self.content, Some((_, 0)))
    }
}
