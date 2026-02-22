//! File-based logger with size-based rotation.
//!
//! Logs are written to `~/.config/mosaico/logs/mosaico.log`. When
//! the file exceeds the configured max size, it is rotated to
//! `mosaico.log.1` (one backup kept).

use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

static LOGGER: OnceLock<Mutex<Logger>> = OnceLock::new();

const LOG_FILE_NAME: &str = "mosaico.log";
const BACKUP_SUFFIX: &str = ".1";

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    /// Whether file logging is enabled. Defaults to `false`.
    pub enabled: bool,
    /// Minimum log level: "debug", "info", "warn", or "error".
    pub level: String,
    /// Maximum log file size in megabytes before rotation.
    pub max_file_mb: u64,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            level: "info".into(),
            max_file_mb: 10,
        }
    }
}

/// Log severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }

    fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "debug" => Self::Debug,
            "warn" => Self::Warn,
            "error" => Self::Error,
            _ => Self::Info,
        }
    }
}

struct Logger {
    file: File,
    path: PathBuf,
    min_level: Level,
    max_bytes: u64,
    written: u64,
}

/// Initialises the global logger. Call once at daemon startup.
///
/// Does nothing if `config.enabled` is `false`.
pub fn init(config: &LogConfig) {
    if !config.enabled {
        return;
    }
    let Some(dir) = crate::config::config_dir() else {
        return;
    };
    let log_dir = dir.join("logs");
    let _ = fs::create_dir_all(&log_dir);
    let path = log_dir.join(LOG_FILE_NAME);

    let file = match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(f) => f,
        Err(_) => return,
    };
    let written = file.metadata().map(|m| m.len()).unwrap_or(0);

    let logger = Logger {
        file,
        path,
        min_level: Level::parse(&config.level),
        max_bytes: config.max_file_mb * 1024 * 1024,
        written,
    };

    let _ = LOGGER.set(Mutex::new(logger));
}

/// Writes a log line if the level is at or above the configured minimum.
pub fn write(level: Level, args: fmt::Arguments<'_>) {
    let Some(mutex) = LOGGER.get() else {
        return;
    };
    let Ok(mut logger) = mutex.lock() else {
        return;
    };
    if level < logger.min_level {
        return;
    }
    let now = timestamp();
    let line = format!("{now} [{lvl}] {args}\n", lvl = level.as_str());
    let bytes = line.len() as u64;

    let _ = logger.file.write_all(line.as_bytes());
    logger.written += bytes;

    if logger.max_bytes > 0 && logger.written >= logger.max_bytes {
        logger.rotate();
    }
}

impl Logger {
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

fn timestamp() -> String {
    // Use std::time for a simple UTC timestamp. No chrono dependency.
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let (h, m, s) = (secs / 3600 % 24, secs / 60 % 60, secs % 60);
    format!("{h:02}:{m:02}:{s:02}")
}

/// Logs at DEBUG level.
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => { $crate::log::write($crate::log::Level::Debug, format_args!($($arg)*)) };
}

/// Logs at INFO level.
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::log::write($crate::log::Level::Info, format_args!($($arg)*)) };
}

/// Logs at WARN level.
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => { $crate::log::write($crate::log::Level::Warn, format_args!($($arg)*)) };
}

/// Logs at ERROR level.
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::log::write($crate::log::Level::Error, format_args!($($arg)*)) };
}
