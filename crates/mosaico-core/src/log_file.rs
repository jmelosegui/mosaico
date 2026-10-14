//! The log file and its rotation.

use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::{Level, LogConfig};

const LOG_FILE_NAME: &str = "mosaico.log";
const BACKUP_SUFFIX: &str = ".1";

/// The open log file.
pub(super) struct Logger {
    file: File,
    path: PathBuf,
    min_level: Level,
    max_bytes: u64,
    written: u64,
}

impl Logger {
    /// Opens, or creates, the log file in `log_dir`, creating the
    /// directory if needed.
    ///
    /// `None` when the file cannot be opened.
    pub(super) fn open(config: &LogConfig, min_level: Level, log_dir: &Path) -> Option<Self> {
        let _ = fs::create_dir_all(log_dir);
        let path = log_dir.join(LOG_FILE_NAME);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .ok()?;
        let written = file.metadata().map(|m| m.len()).unwrap_or(0);

        Some(Self {
            file,
            path,
            min_level,
            max_bytes: config.max_file_mb * 1024 * 1024,
            written,
        })
    }

    /// Writes a line if `level` is at or above the configured minimum.
    pub(super) fn write(&mut self, level: Level, args: fmt::Arguments<'_>) {
        if level >= self.min_level {
            self.append(&line(level, args));
        }
    }

    /// Writes a line that is already formatted, rotating the file when
    /// it has grown too large.
    pub(super) fn append(&mut self, line: &str) {
        let _ = self.file.write_all(line.as_bytes());
        let _ = self.file.flush();
        self.written += line.len() as u64;

        if self.max_bytes > 0 && self.written >= self.max_bytes {
            self.rotate();
        }
    }

    fn rotate(&mut self) {
        let backup = self.path.with_extension(format!(
            "{}{}",
            LOG_FILE_NAME.rsplit('.').next().unwrap_or("log"),
            BACKUP_SUFFIX
        ));
        // Close current file by replacing it with a throwaway.
        let _ = fs::rename(&self.path, &backup);
        if let Ok(f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            self.file = f;
        }
        self.written = 0;
    }
}

/// A line as written to the log: `HH:MM:SS [LEVEL] message`.
pub(super) fn line(level: Level, args: fmt::Arguments<'_>) -> String {
    format!("{} [{}] {args}\n", timestamp(), level.as_str())
}

fn timestamp() -> String {
    // Use std::time for a simple UTC timestamp. No chrono dependency.
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let (h, m, s) = (secs / 3600 % 24, secs / 60 % 60, secs % 60);
    format!("{h:02}:{m:02}:{s:02}")
}
