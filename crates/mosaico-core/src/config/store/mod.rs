mod load;
mod merge_bar;
mod merge_config;
mod merge_keybindings;
mod merge_unfocused;
mod migrate;
mod migrate_borders;
mod migrate_workspaces;

#[cfg(test)]
#[path = "tests/support.rs"]
mod test_support;

use std::fmt;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::{Error, Instance, Result};

use super::report;

/// The config files of one instance, all kept in one directory.
///
/// Every operation reads and writes files in [`dir`](Self::dir) and
/// nowhere else, so a store on a temporary directory exercises loading,
/// merging and migrating without touching the user's files. The daemon
/// and the CLI use [`ConfigStore::for_instance`].
///
/// A missing file is never created: loading it gives the defaults and
/// merging into it does nothing. Two kinds of operation change files
/// that exist: loading `config.toml` rewrites keys from older versions
/// after saving a backup next to it, and the `merge_missing_*` methods
/// add what newer versions introduced, appending it or, for the
/// `unfocused` border color, inserting it into `[borders.colors]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigStore {
    dir: PathBuf,
}

impl ConfigStore {
    /// A store for the config files in `dir`.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// The store of `instance`, on the files in its config directory.
    pub fn for_instance(instance: &Instance) -> Self {
        Self::new(instance.config_dir())
    }

    /// The directory holding the files.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// `config.toml`: theme, layout, workspaces, borders, mouse and logging.
    pub fn config_path(&self) -> PathBuf {
        self.dir.join("config.toml")
    }

    /// `keybindings.toml`: the hotkeys.
    pub fn keybindings_path(&self) -> PathBuf {
        self.dir.join("keybindings.toml")
    }

    /// `rules.toml`: the community window rules the daemon downloads.
    pub fn rules_path(&self) -> PathBuf {
        self.dir.join("rules.toml")
    }

    /// `user-rules.toml`: the user's own window rules.
    pub fn user_rules_path(&self) -> PathBuf {
        self.dir.join("user-rules.toml")
    }

    /// `bar.toml`: the status bar.
    pub fn bar_path(&self) -> PathBuf {
        self.dir.join("bar.toml")
    }
}

/// Loads the file at `path` with `try_load`, falling back to `default`.
///
/// A missing file silently gives the default; any other failure is
/// reported as a warning, on standard error and in the log.
fn load_or_default<T>(
    path: &Path,
    try_load: impl FnOnce() -> Result<T>,
    default: impl Fn() -> T,
) -> T {
    if !path.exists() {
        return default();
    }
    match try_load() {
        Ok(val) => val,
        Err(e) => {
            report::warning(format_args!("{e}"));
            default()
        }
    }
}

/// Reads and parses the config file at `path`.
fn load_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    parse_toml(path, &read_file(path)?)
}

/// Reads the config file at `path`, naming it in the error.
fn read_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| file_error(path, e))
}

/// Parses `content`, the TOML of the config file at `path`, naming the
/// file in the error.
fn parse_toml<T: DeserializeOwned>(path: &Path, content: &str) -> Result<T> {
    toml::from_str(content).map_err(|e| file_error(path, e))
}

/// A config error about the file at `path`. It reads
/// `<path>: <problem>`, as these errors always have.
fn file_error(path: &Path, problem: impl fmt::Display) -> Error {
    Error::config(Some(path.to_path_buf()), problem.to_string())
}

/// Appends `text` to the config file at `path`.
fn append_to_file(path: &Path, text: &str) -> Result<()> {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|e| file_error(path, format_args!("could not open it for appending: {e}")))?;
    f.write_all(text.as_bytes())
        .map_err(|e| file_error(path, format_args!("could not append to it: {e}")))
}

#[cfg(test)]
#[path = "tests/errors_tests.rs"]
mod errors_tests;
#[cfg(test)]
#[path = "tests/paths_tests.rs"]
mod paths_tests;
