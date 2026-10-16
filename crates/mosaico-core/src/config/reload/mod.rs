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
}

/// Decides which config files to reload while the daemon runs, and loads
/// them.
///
/// What a change to each file does:
///
/// - `config.toml` and `bar.toml` are loaded again and reloaded only when
///   they load. A file that does not is logged and skipped, so the config
///   in use stays until the file is fixed.
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

    /// Loads the file at `index` in `files` if it changed.
    fn reload_if_changed(&mut self, index: usize) -> Option<ConfigReload> {
        let (file, seen) = &mut self.files[index];
        let file = *file;
        let modified = modified(&file.path(&self.store));
        if *seen == modified {
            return None;
        }
        *seen = modified;

        match file {
            Watched::Config => match self.store.try_load_config() {
                Ok(config) => {
                    crate::log_info!("config.toml changed, reloading");
                    Some(ConfigReload::Config(Box::new(config)))
                }
                Err(e) => {
                    crate::log_info!("config.toml invalid, skipping: {e}");
                    None
                }
            },
            Watched::UserRules => {
                crate::log_info!("user-rules.toml changed, reloading merged rules");
                Some(ConfigReload::Rules(self.store.load_merged_rules()))
            }
            Watched::Bar => match self.store.try_load_bar() {
                Ok(bar) => {
                    crate::log_info!("bar.toml changed, reloading");
                    Some(ConfigReload::Bar(Box::new(bar)))
                }
                Err(e) => {
                    crate::log_info!("bar.toml invalid, skipping: {e}");
                    None
                }
            },
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
