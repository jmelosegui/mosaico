//! File-based logger with size-based rotation.
//!
//! Logs are written to `logs/mosaico.log` in the instance's config
//! directory. When the file exceeds the configured max size, it is
//! rotated to `mosaico.log.1` (one backup kept).
//!
//! Lines logged before [`init`](crate::log::init) are kept and written
//! when it runs, if logging is enabled. The daemon reads its config, and
//! reports what is wrong with it, before it knows whether to log at all.

use std::fmt;
use std::path::Path;
use std::sync::{Mutex, OnceLock, PoisonError};

use serde::{Deserialize, Serialize};

#[path = "log_backlog.rs"]
mod backlog;
#[path = "log_file.rs"]
mod file;

use backlog::Backlog;
use file::Logger;

/// The logger once [`init`] has run, `None` when logging is off.
static LOGGER: OnceLock<Option<Mutex<Logger>>> = OnceLock::new();

/// Lines logged before [`init`] ran.
static BACKLOG: Mutex<Backlog> = Mutex::new(Backlog::new());

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
    /// Verbose debugging output.
    Debug,
    /// Informational messages.
    Info,
    /// Warnings.
    Warn,
    /// Errors.
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

/// Initialises the global logger, writing to this process's instance's
/// log directory. Call once at daemon startup.
///
/// Does nothing if `config.enabled` is `false`, other than drop the
/// lines logged so far, and the same when the instance of this process
/// has not been resolved, since its log directory is not known.
/// Otherwise the lines logged so far follow its own first line,
/// filtered by the configured level. Only the first call has any
/// effect.
pub fn init(config: &LogConfig) {
    let log_dir = crate::Instance::current().map(|instance| instance.log_dir());
    start(config, log_dir.as_deref());
}

/// Like [`init`], but writes to `log_dir`, so a test can keep its log
/// away from the user's.
pub fn init_in(config: &LogConfig, log_dir: &Path) {
    start(config, Some(log_dir));
}

/// Sets up the logger. Without a directory, logging is off.
fn start(config: &LogConfig, log_dir: Option<&Path>) {
    // Only the first call counts, so a later one must not even create
    // the file.
    if LOGGER.get().is_some() {
        return;
    }
    let min_level = Level::parse(&config.level);
    let logger = match log_dir {
        Some(dir) if config.enabled => Logger::open(config, min_level, dir),
        _ => None,
    };
    let early = {
        let mut backlog = BACKLOG.lock().unwrap_or_else(PoisonError::into_inner);
        if LOGGER.set(logger.map(Mutex::new)).is_err() {
            return;
        }
        backlog.take(min_level)
    };

    write(
        Level::Info,
        format_args!(
            "Logger initialized (level={}, max_mb={})",
            config.level, config.max_file_mb
        ),
    );
    if let Some(Some(logger)) = LOGGER.get()
        && let Ok(mut logger) = logger.lock()
    {
        for line in early {
            logger.append(&line);
        }
    }
}

/// Writes a log line if the level is at or above the configured minimum.
///
/// Until [`init`] has run, the line is kept for it instead.
pub fn write(level: Level, args: fmt::Arguments<'_>) {
    match LOGGER.get() {
        Some(Some(logger)) => {
            if let Ok(mut logger) = logger.lock() {
                logger.write(level, args);
            }
        }
        Some(None) => {}
        None => keep(level, args),
    }
}

/// Keeps a line logged before [`init`].
fn keep(level: Level, args: fmt::Arguments<'_>) {
    let mut backlog = BACKLOG.lock().unwrap_or_else(PoisonError::into_inner);
    // `init` sets the logger while it holds the backlog, so a logger set
    // since `write` looked shows up here.
    if LOGGER.get().is_some() {
        drop(backlog);
        write(level, args);
        return;
    }
    backlog.keep(level, || file::line(level, args));
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
