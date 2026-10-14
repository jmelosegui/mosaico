//! Finding an instance from the environment and the user's directories.

use std::ffi::OsString;
use std::path::PathBuf;

use super::{Instance, InstanceError, validate_name};
use crate::user_dirs::{UserDir, UserDirs};

/// Selects a named instance. Unset or empty means the default instance.
pub const INSTANCE_VAR: &str = "MOSAICO_INSTANCE";
/// Overrides the config directory of the instance.
pub const CONFIG_DIR_VAR: &str = "MOSAICO_CONFIG_DIR";
/// Overrides the data directory (PID file) of the instance.
pub const DATA_DIR_VAR: &str = "MOSAICO_DATA_DIR";

/// Folder of mosaico's files in each of the user's directories.
const APP_DIR: &str = "mosaico";
/// Folder of the named instances inside mosaico's.
const INSTANCES_DIR: &str = "instances";

impl Instance {
    /// Resolves the instance from environment variables read through
    /// `var` and the user's directories as `dirs` reports them, so it can
    /// be tested without touching the real environment.
    ///
    /// - [`INSTANCE_VAR`] picks a named instance; unset or empty is the
    ///   default one.
    /// - [`CONFIG_DIR_VAR`] and [`DATA_DIR_VAR`] override its directories
    ///   and are used as given; empty means unset. `dirs` is only asked
    ///   for a directory that is not overridden.
    /// - Otherwise the default instance uses a `mosaico` folder in each of
    ///   the user's directories, as mosaico always has, and a named
    ///   instance an `instances/<name>` folder inside that, so it shares
    ///   nothing with the default one.
    pub fn from_env(
        var: impl Fn(&str) -> Option<OsString>,
        dirs: &impl UserDirs,
    ) -> Result<Self, InstanceError> {
        let name = var(INSTANCE_VAR)
            .map(|v| v.to_string_lossy().into_owned())
            .filter(|v| !v.is_empty());
        if let Some(name) = &name {
            validate_name(name)?;
        }
        let per_instance = |base: PathBuf| match &name {
            Some(name) => base.join(APP_DIR).join(INSTANCES_DIR).join(name),
            None => base.join(APP_DIR),
        };

        let config_dir = match var(CONFIG_DIR_VAR).filter(|v| !v.is_empty()) {
            Some(dir) => PathBuf::from(dir),
            None => base(dirs.config_dir())
                .map(per_instance)
                .ok_or(InstanceError::NoDirectory("config"))?,
        };
        let data_dir = match var(DATA_DIR_VAR).filter(|v| !v.is_empty()) {
            Some(dir) => PathBuf::from(dir),
            None => base(dirs.data_dir())
                .map(per_instance)
                .ok_or(InstanceError::NoDirectory("data"))?,
        };

        Ok(Self {
            name,
            config_dir,
            data_dir,
        })
    }
}

/// The base directory `dir` gives, or `None` when its setting is not
/// set.
fn base(dir: UserDir) -> Option<PathBuf> {
    let value = dir.value?;
    Some(match dir.subfolder {
        Some(folder) => value.join(folder),
        None => value,
    })
}

#[cfg(test)]
#[path = "instance_resolve_tests.rs"]
mod tests;
