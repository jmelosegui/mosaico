//! The settings the General page edits, and the edits not yet applied.

use mosaico_core::config::{Config, Theme};

/// The levels a log file can start at, least severe first.
pub const LOG_LEVELS: [&str; 4] = ["debug", "info", "warn", "error"];

/// The sizes, in megabytes, a log file may grow to before it is rotated.
pub const LOG_SIZES_MB: std::ops::RangeInclusive<u64> = 1..=1024;

/// What the General page shows.
#[derive(Debug, Clone, PartialEq)]
pub struct General {
    pub theme: Theme,
    pub start_at_sign_in: bool,
    pub log_to_file: bool,
    pub log_level: String,
    pub max_log_mb: u64,
}

impl General {
    /// The General settings of `config`, with whether mosaico starts at
    /// sign-in, which is not part of the config files.
    pub fn from_config(config: &Config, start_at_sign_in: bool) -> Self {
        Self {
            theme: config.theme.resolve(),
            start_at_sign_in,
            log_to_file: config.logging.enabled,
            log_level: config.logging.level.clone(),
            max_log_mb: config.logging.max_file_mb,
        }
    }

    /// What a new install has.
    pub fn defaults() -> Self {
        Self::from_config(&Config::default(), false)
    }

    /// How many settings differ from `other`. The theme and its flavor
    /// count apart.
    pub fn changes_from(&self, other: &Self) -> usize {
        [
            self.theme.name() != other.theme.name(),
            self.theme.flavor_name() != other.theme.flavor_name(),
            self.start_at_sign_in != other.start_at_sign_in,
            self.log_to_file != other.log_to_file,
            self.log_level != other.log_level,
            self.max_log_mb != other.max_log_mb,
        ]
        .into_iter()
        .filter(|&changed| changed)
        .count()
    }
}

/// Settings as they were loaded, and as they are being edited.
#[derive(Debug, Clone)]
pub struct Draft<T> {
    saved: T,
    /// The settings with the edits made so far.
    pub edit: T,
}

impl<T: Clone> Draft<T> {
    /// A draft with no edits.
    pub fn new(saved: T) -> Self {
        Self {
            edit: saved.clone(),
            saved,
        }
    }

    /// The settings as they were loaded.
    pub fn saved(&self) -> &T {
        &self.saved
    }

    /// Drops every edit.
    pub fn revert(&mut self) {
        self.edit = self.saved.clone();
    }
}

impl Draft<General> {
    /// How many settings were edited.
    pub fn changes(&self) -> usize {
        self.edit.changes_from(&self.saved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mosaico_core::config::{CatppuccinFlavor, RosePineFlavor};

    #[test]
    fn each_edited_setting_is_one_change_and_revert_drops_them() {
        // Arrange
        let mut draft = Draft::new(General::defaults());

        // Act
        draft.edit.log_to_file = true;
        draft.edit.max_log_mb = 20;
        let changes = draft.changes();
        draft.revert();

        // Assert
        assert_eq!(changes, 2);
        assert_eq!(draft.changes(), 0);
        assert_eq!(draft.edit, General::defaults());
    }

    #[test]
    fn another_theme_family_changes_the_theme_and_the_flavor() {
        let mut draft = Draft::new(General::defaults());
        draft.edit.theme = Theme::Catppuccin(CatppuccinFlavor::Latte);
        assert_eq!(draft.changes(), 1);
        draft.edit.theme = Theme::RosePine(RosePineFlavor::Dawn);
        assert_eq!(draft.changes(), 2);
    }

    #[test]
    fn the_defaults_are_those_of_a_new_install() {
        let defaults = General::defaults();
        assert_eq!(defaults.theme, Theme::Catppuccin(CatppuccinFlavor::Mocha));
        assert!(!defaults.log_to_file);
        assert!(LOG_LEVELS.contains(&defaults.log_level.as_str()));
        assert!(LOG_SIZES_MB.contains(&defaults.max_log_mb));
    }
}
