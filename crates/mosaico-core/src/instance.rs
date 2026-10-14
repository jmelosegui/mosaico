//! Which daemon a process belongs to.
//!
//! Everything that tells one daemon apart from another (its IPC endpoint,
//! its single instance lock, its PID file, its config and log directory)
//! is derived from an [`Instance`]. The default instance is the user's
//! own and keeps the names and paths mosaico has always used. A named
//! instance gets its own of each, which is what lets tests run a daemon
//! without touching the user's daemon or config.

use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Selects a named instance. Unset or empty means the default instance.
pub const INSTANCE_VAR: &str = "MOSAICO_INSTANCE";
/// Overrides the config directory of the instance.
pub const CONFIG_DIR_VAR: &str = "MOSAICO_CONFIG_DIR";
/// Overrides the data directory (PID file) of the instance.
pub const DATA_DIR_VAR: &str = "MOSAICO_DATA_DIR";

/// Longest instance name accepted.
const MAX_NAME_LEN: usize = 64;

/// One daemon and the files it owns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    name: Option<String>,
    config_dir: PathBuf,
    data_dir: PathBuf,
}

/// Why an instance could not be resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceError {
    /// The name is empty, too long, or has characters other than ASCII
    /// letters, digits, `-` and `_`.
    InvalidName(String),
    /// A directory was not given and has no default on this system.
    NoDirectory(&'static str),
}

impl fmt::Display for InstanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName(name) => write!(
                f,
                "invalid {INSTANCE_VAR} {name:?}: use 1-{MAX_NAME_LEN} ASCII letters, digits, '-' or '_'"
            ),
            Self::NoDirectory(what) => write!(f, "could not determine the {what} directory"),
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

    /// Resolves the instance from environment variables read through
    /// `var`, so it can be tested without touching the real environment.
    ///
    /// - [`INSTANCE_VAR`] picks a named instance; unset or empty is the
    ///   default one.
    /// - [`CONFIG_DIR_VAR`] and [`DATA_DIR_VAR`] override its directories.
    /// - Without overrides, the default instance uses `~/.config/mosaico`
    ///   and the local app data `mosaico` folder, as mosaico always has,
    ///   and a named instance uses an `instances/<name>` folder inside
    ///   each, so it shares nothing with the default one.
    pub fn from_env(var: impl Fn(&str) -> Option<OsString>) -> Result<Self, InstanceError> {
        let name = var(INSTANCE_VAR)
            .map(|v| v.to_string_lossy().into_owned())
            .filter(|v| !v.is_empty());
        if let Some(name) = &name {
            validate_name(name)?;
        }
        let per_instance = |base: PathBuf| match &name {
            Some(name) => base.join("instances").join(name),
            None => base,
        };

        let config_dir = match var(CONFIG_DIR_VAR).filter(|v| !v.is_empty()) {
            Some(dir) => PathBuf::from(dir),
            None => var("USERPROFILE")
                .or_else(|| var("HOME"))
                .map(|home| per_instance(PathBuf::from(home).join(".config").join("mosaico")))
                .ok_or(InstanceError::NoDirectory("config"))?,
        };
        let data_dir = match var(DATA_DIR_VAR).filter(|v| !v.is_empty()) {
            Some(dir) => PathBuf::from(dir),
            None => var("LOCALAPPDATA")
                .map(|base| per_instance(PathBuf::from(base).join("mosaico")))
                .ok_or(InstanceError::NoDirectory("data"))?,
        };

        Ok(Self {
            name,
            config_dir,
            data_dir,
        })
    }

    /// The instance of this process, resolved from the environment the
    /// first time it is asked for.
    ///
    /// The CLI checks this at startup and refuses to run on an error, so
    /// a mistyped instance name can never fall back to the user's daemon.
    pub fn current() -> Result<&'static Instance, InstanceError> {
        static CURRENT: OnceLock<Result<Instance, InstanceError>> = OnceLock::new();
        CURRENT
            .get_or_init(|| Self::from_env(|key| std::env::var_os(key)))
            .as_ref()
            .map_err(Clone::clone)
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
