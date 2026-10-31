mod run;

#[cfg(test)]
#[path = "tests/support.rs"]
mod test_support;

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::{BarConfig, Config, ConfigStore, WindowRule};

pub use run::ConfigWatch;

/// A validated config reload ready to be applied.
#[derive(Debug, Clone)]
pub enum ConfigReload {
    /// Layout and border settings changed. Boxed because `Config` is
    /// substantially larger than the other variants.
    Config(Box<Config>),
    /// Window rules changed: the whole rule set, user rules first, as
    /// [`ConfigStore::load_merged_rules`] returns it.
    Rules(Vec<WindowRule>),
    /// Bar configuration changed.
    Bar(Box<BarConfig>),
    /// `file` changed but does not load, for `reason`, so what is in use
    /// stays. Reported so the user hears of it, not only the log.
    Invalid {
        /// The file's name, such as `config.toml`.
        file: &'static str,
        /// Why it does not load, without the file's path.
        reason: String,
    },
}

/// Decides which config files to reload while the daemon runs, and loads
/// them.
///
/// What a change to each file does:
///
/// - `config.toml` and `bar.toml` are loaded again and reloaded only when
///   they load. A file that does not is reported as
///   [`ConfigReload::Invalid`] and skipped, so the config in use stays
///   until the file is fixed. One that was deleted or cannot be read is
///   skipped without a report, since some editors delete a file before
///   they write it again or hold it while they write it, and the write
///   that ends it is a change of its own.
/// - `user-rules.toml` reloads the whole rule set, user rules first. A
///   file that does not load counts as no user rules, as at startup.
/// - `rules.toml` is not watched. It holds the community rules, which the
///   daemon downloads at startup and reloads itself, and the next download
///   overwrites hand edits anyway. Edit `user-rules.toml` instead.
/// - `keybindings.toml` is not watched: hotkeys are registered once, at
///   startup, so a change needs a restart.
///
/// A file has changed when its modification time differs from the last
/// one seen, including when the file appears or disappears. Content alone
/// is never compared, and nothing waits for a burst of writes to settle:
/// each change is loaded as soon as it is seen. [`ConfigWatch`] checks
/// the files each time their directory changes.
#[derive(Debug)]
pub struct ConfigReloader {
    store: ConfigStore,
    /// Each watched file with the modification time last seen, in the
    /// order they are checked.
    files: [(Watched, Option<SystemTime>); 3],
}

/// A file that is reloaded when it changes.
#[derive(Debug, Clone, Copy)]
enum Watched {
    Config,
    UserRules,
    Bar,
}

impl Watched {
    /// Every watched file, in the order they are checked.
    const ALL: [Self; 3] = [Self::Config, Self::UserRules, Self::Bar];

    fn path(self, store: &ConfigStore) -> PathBuf {
        match self {
            Self::Config => store.config_path(),
            Self::UserRules => store.user_rules_path(),
            Self::Bar => store.bar_path(),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Config => "config.toml",
            Self::UserRules => "user-rules.toml",
            Self::Bar => "bar.toml",
        }
    }
}

impl ConfigReloader {
    /// Watches the files in `store` from how they are now: only later
    /// changes are reloaded.
    pub fn new(store: ConfigStore) -> Self {
        let files = Watched::ALL.map(|file| (file, modified(&file.path(&store))));
        Self { store, files }
    }

    /// Loads each watched file that changed since it was last seen, in the
    /// order `config.toml`, `user-rules.toml`, `bar.toml`.
    ///
    /// Files are checked as the iterator reaches them, so a file it does
    /// not reach is still pending on the next call. A change counts as
    /// seen before the file is loaded: a file that does not load is not
    /// tried again until it changes again.
    pub fn reload_changed(&mut self) -> impl Iterator<Item = ConfigReload> + '_ {
        (0..self.files.len()).filter_map(move |index| self.reload_if_changed(index))
    }

    /// Loads every watched file now, changed or not, in the order
    /// `config.toml`, `user-rules.toml`, `bar.toml`, as when the user asks
    /// for the config to be reloaded. Each counts as seen as it is now.
    pub fn load_all(&mut self) -> Vec<ConfigReload> {
        for (file, seen) in &mut self.files {
            *seen = modified(&file.path(&self.store));
        }
        Watched::ALL
            .into_iter()
            .filter_map(|file| self.load(file))
            .collect()
    }

    /// Loads the file at `index` in `files` if it changed.
    fn reload_if_changed(&mut self, index: usize) -> Option<ConfigReload> {
        let (file, seen) = &mut self.files[index];
        let file = *file;
        let modified = modified(&file.path(&self.store));
        if *seen == modified {
            return None;
        }
        *seen = modified;
        crate::log_info!("{} changed", file.name());
        self.load(file)
    }

    /// Loads `file`, or reports why it does not load. A file that cannot
    /// be read, before or after it fails to load, is skipped instead, but
    /// for the user rules, whose absence still reloads the rules.
    fn load(&self, file: Watched) -> Option<ConfigReload> {
        let path = file.path(&self.store);
        if !matches!(file, Watched::UserRules) && !readable(file, &path) {
            return None;
        }
        let loaded = match file {
            Watched::Config => self
                .store
                .try_load_config()
                .map(|config| ConfigReload::Config(Box::new(config))),
            Watched::UserRules => Ok(ConfigReload::Rules(self.store.load_merged_rules())),
            Watched::Bar => self
                .store
                .try_load_bar()
                .map(|bar| ConfigReload::Bar(Box::new(bar))),
        };
        match loaded {
            Ok(reload) => {
                crate::log_info!("Reloading {}", file.name());
                Some(reload)
            }
            Err(_) if !readable(file, &path) => None,
            Err(e) => {
                crate::log_warn!("{} does not load, keeping what is in use: {e}", file.name());
                let reason = match e {
                    crate::Error::Config { message, .. } => message,
                    other => other.to_string(),
                };
                Some(ConfigReload::Invalid {
                    file: file.name(),
                    reason,
                })
            }
        }
    }
}

/// Whether `file`, at `path`, can be read. One that cannot is logged,
/// unless it does not exist.
fn readable(file: Watched, path: &Path) -> bool {
    match std::fs::read(path) {
        Ok(_) => true,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(e) => {
            crate::log_warn!(
                "{} could not be read, keeping what is in use: {e}",
                file.name()
            );
            false
        }
    }
}

/// When the file at `path` was last modified, or `None` when that cannot
/// be read, as for a file that does not exist.
fn modified(path: &Path) -> Option<SystemTime> {
    path.metadata().ok().and_then(|m| m.modified().ok())
}

#[cfg(test)]
#[path = "tests/reload_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/rules_tests.rs"]
mod rules_tests;
