use crate::Result;
use crate::config::bar::BarConfig;
use crate::config::keybinding;
use crate::config::rules::{KeybindingsFile, RulesFile, UserRulesFile};
use crate::config::{Config, Keybinding, WindowRule, default_rules};

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
    /// falling back to the defaults.
    ///
    /// A missing file silently gives the defaults; any other failure is
    /// reported as a warning, on standard error and in the log.
    pub fn load_config(&self) -> Config {
        load_or_default(
            &self.config_path(),
            || self.try_load_config(),
            Config::default,
        )
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

    /// Loads and merges both rule sets: user rules first, then community rules.
    ///
    /// User rules are prepended so they take priority (first match wins
    /// in [`should_manage`](crate::config::should_manage)). Falls back
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
#[path = "tests/load_tests.rs"]
mod tests;
