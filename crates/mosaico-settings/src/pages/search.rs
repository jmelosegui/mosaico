//! What a search of the window matches: a row's label, help and key.

use super::Page;
use crate::model::Settings;

/// A setting's label and help, and the key it has in its file, which
/// search matches.
#[derive(Debug, Clone, Copy)]
pub struct Row {
    pub label: &'static str,
    help: &'static str,
    key: &'static str,
}

impl Row {
    pub const fn new(label: &'static str, help: &'static str) -> Self {
        Self::keyed("", label, help)
    }

    /// The row of the setting at `key` in its file.
    pub const fn keyed(key: &'static str, label: &'static str, help: &'static str) -> Self {
        Self { label, help, key }
    }

    /// The help under the label, if the row has any.
    pub fn help(&self) -> Option<&'static str> {
        (!self.help.is_empty()).then_some(self.help)
    }

    /// Whether the label, the help or the key contains `query`, ignoring
    /// case.
    pub fn matches(&self, query: &str) -> bool {
        let query = query.trim().to_lowercase();
        [self.label, self.help, self.key]
            .iter()
            .any(|text| text.to_lowercase().contains(&query))
    }
}

/// How many rows of a page of `config.toml` settings match `query`, of
/// the sections that apply under `settings`.
pub fn count(page: Page, query: &str, settings: &Settings) -> usize {
    page.sections().map_or(0, |sections| {
        sections
            .iter()
            .filter(|(title, _)| super::applies(title, settings))
            .flat_map(|(_, entries)| entries.iter())
            .filter(|entry| entry.row().matches(query))
            .count()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mosaico_core::config::schema::SettingValue;

    #[test]
    fn a_row_matches_its_key() {
        let row = Row::keyed("borders.corner_style", "Corners", "How round they are.");

        assert!(row.matches("corner_style"));
        assert!(row.matches(" ROUND "));
        assert!(!row.matches("gap"));
    }

    #[test]
    fn counts_the_rows_of_a_page_that_match() {
        let settings = Settings::defaults();

        assert!(count(Page::Layout, "gap", &settings) > 0);
        assert_eq!(count(Page::Layout, "zzz", &settings), 0);
        assert_eq!(count(Page::Bar, "gap", &settings), 0);
    }

    #[test]
    fn the_space_kept_for_another_bar_is_only_found_while_that_bar_is_picked() {
        // Arrange
        let mosaico = Settings::defaults();
        let mut external = Settings::defaults();
        external.set("bar.provider", SettingValue::Text("external".into()));

        // Act
        let under_mosaico = count(Page::Bar, "reserve", &mosaico);
        let under_external = count(Page::Bar, "reserve", &external);

        // Assert
        assert_eq!(under_mosaico, 0);
        assert_eq!(under_external, 4);
    }
}
