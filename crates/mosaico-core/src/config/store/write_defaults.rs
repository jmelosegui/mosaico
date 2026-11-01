//! Writing the default config files an instance does not have yet.

use std::io;
use std::path::PathBuf;

use crate::config::template;

use super::ConfigStore;

/// What [`ConfigStore::write_missing_defaults`] did with one file.
#[derive(Debug)]
pub enum DefaultFile {
    /// The file was missing and now holds the default.
    Created(PathBuf),
    /// The file was there already and was left alone.
    Existed(PathBuf),
    /// The file was missing and could not be written.
    Failed(PathBuf, io::Error),
}

impl ConfigStore {
    /// Creates the store's directory when it does not exist, and writes
    /// each config file that is missing with its commented default:
    /// `config.toml`, `keybindings.toml`, `rules.toml`, `user-rules.toml`
    /// and `bar.toml`, in that order. A file that exists is never
    /// touched. Returns what happened to each file, in the same order.
    ///
    /// # Errors
    ///
    /// Fails, writing nothing, when the directory cannot be created.
    pub fn write_missing_defaults(&self) -> io::Result<Vec<DefaultFile>> {
        std::fs::create_dir_all(self.dir())?;
        let files = [
            (
                self.config_path(),
                template::generate_config as fn() -> String,
            ),
            (self.keybindings_path(), template::generate_keybindings),
            (self.rules_path(), template::generate_rules),
            (self.user_rules_path(), template::generate_user_rules),
            (self.bar_path(), template::generate_bar),
        ];
        Ok(files
            .into_iter()
            .map(|(path, generate)| {
                if path.exists() {
                    return DefaultFile::Existed(path);
                }
                match std::fs::write(&path, generate()) {
                    Ok(()) => DefaultFile::Created(path),
                    Err(e) => DefaultFile::Failed(path, e),
                }
            })
            .collect())
    }
}

#[cfg(test)]
#[path = "tests/write_defaults_tests.rs"]
mod tests;
