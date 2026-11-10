use crate::Result;
use crate::config::bar::BarConfig;
use crate::config::bar_section::read_bar_section;
use crate::config::keybinding;
use crate::config::report;
use crate::config::rules::{KeybindingsFile, RulesFile, UserRulesFile};
use crate::config::{
    BarSection, Config, Keybinding, LaunchBinding, ProviderCheck, WindowRule, count_manage_rules,
    default_rules, parse_launches,
};

use super::migrate::migrate_config_if_needed;
use super::{ConfigStore, load_or_default, load_toml, parse_toml, read_file};

impl ConfigStore {
    /// Loads and parses `config.toml`, then clamps its values with
    /// [`Config::validate`].
    ///
    /// If the file still has keys from older versions
    /// (`[layout.workspaces]`, or `focused` / `monocle` directly under
    /// `[borders]`), it is backed up and rewritten in place to the
    /// current shape before parsing.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`](crate::Error::Config) naming the file
    /// if it cannot be read, cannot be migrated, or is not valid TOML for
    /// a [`Config`].
    pub fn try_load_config(&self) -> Result<Config> {
        let path = self.config_path();
        let content = read_file(&path)?;

        let content = match migrate_config_if_needed(&path, &content)? {
            Some(rewritten) => rewritten,
            None => content,
        };

        let mut config: Config = parse_toml(&path, &content)?;
        config.validate();
        Ok(config)
    }

    /// Loads `config.toml` like [`try_load_config`](Self::try_load_config),
    /// falling back to the defaults, validated as a loaded file is, so
    /// their border colors come from the theme.
    ///
    /// A missing file silently gives the defaults; any other failure is
    /// reported as a warning, on standard error and in the log. A file
    /// that does not load keeps its `[bar]` table when that can be read
    /// on its own, see [`bar_provider`](Self::bar_provider).
    pub fn load_config(&self) -> Config {
        load_or_default(
            &self.config_path(),
            || self.try_load_config(),
            || {
                let mut config = Config {
                    bar: self.bar_section(),
                    ..Config::default()
                };
                config.validate();
                config
            },
        )
    }

    /// Which bar `config.toml` picks, read from its `[bar]` table alone,
    /// so a mistake elsewhere in the file does not change the bar.
    ///
    /// A missing file, or one without the table, picks the built-in bar.
    /// A table that cannot be read, in a file that cannot be read or is
    /// not TOML, is [`ProviderCheck::Unreadable`]: the built-in bar never
    /// draws over a bar the user may have picked.
    pub fn bar_provider(&self) -> ProviderCheck {
        match self.read_bar_section() {
            Some(section) => section.check(),
            None => ProviderCheck::Unreadable,
        }
    }

    /// The `[bar]` table [`bar_provider`](Self::bar_provider) reads, or
    /// the one an unreadable table stands for.
    pub fn bar_section(&self) -> BarSection {
        self.read_bar_section()
            .unwrap_or_else(BarSection::unreadable)
    }

    fn read_bar_section(&self) -> Option<BarSection> {
        match std::fs::read_to_string(self.config_path()) {
            Ok(content) => read_bar_section(&content),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Some(BarSection::default()),
            Err(_) => None,
        }
    }

    /// Loads and parses `keybindings.toml`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`](crate::Error::Config) naming the file
    /// if it cannot be read or is not valid TOML for a keybindings file.
    pub fn try_load_keybindings(&self) -> Result<Vec<Keybinding>> {
        let file: KeybindingsFile = load_toml(&self.keybindings_path())?;
        Ok(file.keybinding)
    }

    /// Loads `keybindings.toml`, falling back to the built-in defaults.
    ///
    /// A missing file silently gives the defaults; any other failure is
    /// reported as a warning, on standard error and in the log.
    pub fn load_keybindings(&self) -> Vec<Keybinding> {
        load_or_default(
            &self.keybindings_path(),
            || self.try_load_keybindings(),
            keybinding::defaults,
        )
    }

    /// Reads the `[[launch]]` entries of `keybindings.toml`, each on its
    /// own, as [`parse_launches`] does.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`](crate::Error::Config) naming the file
    /// if it cannot be read, is not TOML, or `launch` is not a list of
    /// tables.
    pub fn try_load_launches(&self) -> Result<Vec<Result<LaunchBinding>>> {
        let path = self.keybindings_path();
        parse_launches(&path, &read_file(&path)?)
    }

    /// Loads the launch bindings in `keybindings.toml`, skipping the
    /// entries that cannot be read.
    ///
    /// A missing file silently gives none. A file that cannot be read
    /// gives none, and each entry skipped is reported as a warning, on
    /// standard error and in the log.
    pub fn load_launches(&self) -> Vec<LaunchBinding> {
        let entries = load_or_default(
            &self.keybindings_path(),
            || self.try_load_launches(),
            Vec::new,
        );
        entries
            .into_iter()
            .filter_map(|entry| {
                entry
                    .inspect_err(|e| report::warning(format_args!("{e}")))
                    .ok()
            })
            .collect()
    }

    /// Loads and parses `rules.toml`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`](crate::Error::Config) naming the file
    /// if it cannot be read or is not valid TOML for a rules file.
    pub fn try_load_rules(&self) -> Result<Vec<WindowRule>> {
        let file: RulesFile = load_toml(&self.rules_path())?;
        Ok(file.rule)
    }

    /// Loads `rules.toml`, falling back to the built-in defaults.
    ///
    /// A missing file silently gives the defaults; any other failure is
    /// reported as a warning, on standard error and in the log.
    pub fn load_rules(&self) -> Vec<WindowRule> {
        load_or_default(&self.rules_path(), || self.try_load_rules(), default_rules)
    }

    /// Loads and parses `user-rules.toml`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`](crate::Error::Config) naming the file
    /// if it cannot be read or is not valid TOML for a rules file.
    pub fn try_load_user_rules(&self) -> Result<Vec<WindowRule>> {
        let file: UserRulesFile = load_toml(&self.user_rules_path())?;
        Ok(file.rule)
    }

    /// Loads `user-rules.toml`, falling back to no rules.
    ///
    /// A missing file silently gives no rules; any other failure is
    /// reported as a warning, on standard error and in the log.
    pub fn load_user_rules(&self) -> Vec<WindowRule> {
        load_or_default(
            &self.user_rules_path(),
            || self.try_load_user_rules(),
            Vec::new,
        )
    }

    /// How many rules of `user-rules.toml` still use `manage` instead of
    /// `action`. A missing or unreadable file has none.
    pub fn user_rules_using_manage(&self) -> usize {
        std::fs::read_to_string(self.user_rules_path())
            .map_or(0, |content| count_manage_rules(&content))
    }

    /// Loads and merges both rule sets: user rules first, then community rules.
    ///
    /// User rules are prepended so they take priority (first match wins
    /// in [`matching_rule`](crate::config::matching_rule)). Falls back
    /// gracefully if either file is missing or invalid.
    pub fn load_merged_rules(&self) -> Vec<WindowRule> {
        let mut rules = self.load_user_rules();
        rules.extend(self.load_rules());
        rules
    }

    /// Loads and parses `bar.toml`, then clamps its values with
    /// [`BarConfig::validate`].
    ///
    /// Colors are **not** resolved here; the caller must call
    /// `resolve_colors(theme)` with the global theme from `config.toml`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::Config`](crate::Error::Config) naming the file
    /// if it cannot be read or is not valid TOML for a [`BarConfig`].
    pub fn try_load_bar(&self) -> Result<BarConfig> {
        let mut config: BarConfig = load_toml(&self.bar_path())?;
        config.validate();
        Ok(config)
    }

    /// Loads `bar.toml`, falling back to the defaults.
    ///
    /// A missing file silently gives the defaults; any other failure is
    /// reported as a warning, on standard error and in the log.
    pub fn load_bar(&self) -> BarConfig {
        load_or_default(&self.bar_path(), || self.try_load_bar(), BarConfig::default)
    }
}

#[cfg(test)]
#[path = "tests/bar_provider_tests.rs"]
mod bar_provider_tests;
#[cfg(test)]
#[path = "tests/defaults_tests.rs"]
mod defaults_tests;
#[cfg(test)]
#[path = "tests/load_tests.rs"]
mod tests;
