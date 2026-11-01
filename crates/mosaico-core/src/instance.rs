//! Which daemon a process belongs to.
//!
//! Everything that tells one daemon apart from another (its IPC endpoint,
//! its single instance lock, its PID file, its config and log directory)
//! is derived from an [`Instance`]. The default instance is the user's
//! own and keeps the names and paths mosaico has always used. A named
//! instance gets its own of each, which is what lets tests run a daemon
//! without touching the user's daemon or config.

use std::fmt;
use std::path::{Path, PathBuf};

use crate::user_dirs::Unusable;

#[path = "instance_current.rs"]
mod current;
#[path = "instance_resolve.rs"]
mod resolve;

pub use resolve::{CONFIG_DIR_VAR, DATA_DIR_VAR, INSTANCE_VAR};

/// Longest instance name accepted.
const MAX_NAME_LEN: usize = 64;

/// One daemon and the files it owns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    name: Option<String>,
    config_dir: PathBuf,
    data_dir: PathBuf,
}

/// Why an instance could not be resolved, or could not be used where the
/// instance of this process is needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceError {
    /// The name is empty, too long, or has characters other than ASCII
    /// letters, digits, `-` and `_`.
    InvalidName(String),
    /// A directory was not given, and the setting the platform keeps it
    /// in does not hold one mosaico can use.
    NoDirectory {
        /// Which directory: `"config"` or `"data"`.
        dir: &'static str,
        /// The variable that gives the directory instead.
        override_var: &'static str,
        /// What is wrong with the platform's setting.
        reason: Unusable,
    },
    /// The instance given is not the one this process resolved at
    /// startup, or the process has not resolved one.
    NotCurrent,
}

impl fmt::Display for InstanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName(name) => write!(
                f,
                "invalid {INSTANCE_VAR} {name:?}: use 1-{MAX_NAME_LEN} ASCII letters, digits, '-' or '_'"
            ),
            Self::NoDirectory {
                dir,
                override_var,
                reason,
            } => write!(
                f,
                "could not determine the {dir} directory: {reason}; set {override_var} to choose one"
            ),
            Self::NotCurrent => {
                f.write_str("the instance given is not the one this process resolved at startup")
            }
        }
    }
}

impl std::error::Error for InstanceError {}

impl Instance {
    /// An instance with explicit directories. `name` is `None` for the
    /// default instance.
    pub fn new(
        name: Option<&str>,
        config_dir: impl Into<PathBuf>,
        data_dir: impl Into<PathBuf>,
    ) -> Result<Self, InstanceError> {
        if let Some(name) = name {
            validate_name(name)?;
        }
        Ok(Self {
            name: name.map(str::to_string),
            config_dir: config_dir.into(),
            data_dir: data_dir.into(),
        })
    }

    /// The instance name, or `None` for the user's default instance.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Whether this is the user's default instance.
    pub fn is_default(&self) -> bool {
        self.name.is_none()
    }

    /// A short identifier for naming the instance's IPC endpoint and
    /// lock: `mosaico` for the default instance, `mosaico-<name>` for a
    /// named one. Only ASCII letters, digits, `-` and `_`.
    pub fn endpoint_name(&self) -> String {
        match &self.name {
            Some(name) => format!("mosaico-{name}"),
            None => "mosaico".to_string(),
        }
    }

    /// Directory holding the configuration files.
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Directory holding runtime state such as the PID file.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Directory the daemon writes its logs to.
    pub fn log_dir(&self) -> PathBuf {
        self.config_dir.join("logs")
    }

    /// Path of the daemon's log file, in [`Self::log_dir`].
    pub fn log_path(&self) -> PathBuf {
        self.log_dir().join(crate::log::file::LOG_FILE_NAME)
    }

    /// Path of the daemon's PID file.
    pub fn pid_path(&self) -> PathBuf {
        self.data_dir.join("mosaico.pid")
    }
}

fn validate_name(name: &str) -> Result<(), InstanceError> {
    let valid = !name.is_empty()
        && name.len() <= MAX_NAME_LEN
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if valid {
        Ok(())
    } else {
        Err(InstanceError::InvalidName(name.to_string()))
    }
}

#[cfg(test)]
#[path = "instance_tests.rs"]
mod tests;
