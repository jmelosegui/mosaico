//! Remembering, while an installer replaces mosaico, that the daemon was
//! running, so the new version starts it again.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::Result;

/// A file whose presence says the daemon was stopped for an installer and
/// should start again once the installer is done.
///
/// The executable that stops the daemon is the old version and the one
/// that starts it again is the new one, in separate processes, so the
/// note has to outlive both. Each instance has its own, at
/// [`Instance::restart_marker_path`](crate::Instance::restart_marker_path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestartMarker {
    path: PathBuf,
}

impl RestartMarker {
    /// A marker at `path`.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Where the marker is.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Notes that the daemon should start again, creating the directory
    /// if needed.
    pub fn set(&self) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(&self.path, "")?;
        Ok(())
    }

    /// Removes the note, and says whether there was one.
    pub fn take(&self) -> Result<bool> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_marker(name: &str) -> RestartMarker {
        let dir = std::env::temp_dir()
            .join("mosaico-restart-tests")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        RestartMarker::new(dir.join("nested").join("restart"))
    }

    #[test]
    fn taking_a_marker_that_was_never_set_finds_none() {
        // Arrange
        let marker = temp_marker("unset");

        // Act
        let taken = marker.take().unwrap();

        // Assert
        assert!(!taken);
    }

    #[test]
    fn a_set_marker_is_taken_once() {
        // Arrange
        let marker = temp_marker("set");

        // Act
        marker.set().unwrap();
        let first = marker.take().unwrap();
        let second = marker.take().unwrap();

        // Assert
        assert!(first);
        assert!(!second);
        assert!(!marker.path().exists());
    }
}
