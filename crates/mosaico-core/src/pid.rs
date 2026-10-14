use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::Result;

/// The file a running daemon records its process id in.
///
/// The CLI uses it to tell a daemon that stopped answering apart from
/// one that is gone, and to end a stuck daemon. Each instance has its
/// own, at [`Instance::pid_path`](crate::Instance::pid_path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PidFile {
    path: PathBuf,
}

impl PidFile {
    /// A PID file at `path`.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Where the file is.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Records `pid`, creating the directory if needed.
    pub fn write(&self, pid: u32) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(&self.path, pid.to_string())?;
        Ok(())
    }

    /// The recorded process id, or `None` when there is no file. A file
    /// that does not hold a process id is an [`io::ErrorKind::InvalidData`]
    /// error.
    pub fn read(&self) -> Result<Option<u32>> {
        if !self.path.exists() {
            return Ok(None);
        }
        let contents = fs::read_to_string(&self.path)?;
        let pid: u32 = contents.trim().parse().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid PID file contents: {e}"),
            )
        })?;
        Ok(Some(pid))
    }

    /// Deletes the file, if there is one.
    pub fn remove(&self) -> Result<()> {
        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str) -> PidFile {
        let dir = std::env::temp_dir()
            .join("mosaico-pid-tests")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        PidFile::new(dir.join("nested").join("mosaico.pid"))
    }

    #[test]
    fn a_missing_file_reads_as_none() {
        // Arrange
        let file = temp_file("missing");

        // Act / Assert
        assert_eq!(file.read().unwrap(), None);
        assert!(file.remove().is_ok(), "removing nothing is fine");
    }

    #[test]
    fn a_written_pid_reads_back_and_can_be_removed() {
        // Arrange
        let file = temp_file("round-trip");

        // Act
        file.write(4242).unwrap();
        let read = file.read().unwrap();
        file.remove().unwrap();

        // Assert
        assert_eq!(read, Some(4242));
        assert_eq!(file.read().unwrap(), None);
    }

    #[test]
    fn garbage_in_the_file_is_an_error() {
        // Arrange
        let file = temp_file("garbage");
        file.write(1).unwrap();
        fs::write(file.path(), "not a pid").unwrap();

        // Act / Assert
        assert!(file.read().is_err());
    }
}
