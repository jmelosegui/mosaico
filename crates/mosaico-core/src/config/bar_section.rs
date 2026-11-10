//! `[bar]` in `config.toml`: which bar shows mosaico.
//!
//! It lives in `config.toml` rather than `bar.toml` because it decides
//! whether `bar.toml` is read at all.

use serde::{Deserialize, Serialize};

/// The bar that shows mosaico, as `[bar] provider` names it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BarChoice {
    /// mosaico's own bar, styled in `bar.toml`.
    #[default]
    Mosaico,
    /// Whichever bar connects through the bar API. mosaico draws none.
    External,
    /// No bar, and no space kept for one.
    Off,
}

impl BarChoice {
    /// Every choice, in the order the settings offer them.
    pub const ALL: [Self; 3] = [Self::Mosaico, Self::External, Self::Off];

    /// The name `[bar] provider` gives it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Mosaico => "mosaico",
            Self::External => "external",
            Self::Off => "none",
        }
    }

    /// The choice `name` stands for, compared without regard to case, or
    /// `None` for a name mosaico does not know.
    pub fn parse(name: &str) -> Option<Self> {
        let name = name.trim();
        Self::ALL
            .into_iter()
            .find(|choice| choice.name().eq_ignore_ascii_case(name))
    }

    /// Whether mosaico draws the bar itself, and so reads `bar.toml`.
    pub fn is_builtin(self) -> bool {
        self == Self::Mosaico
    }
}

/// The `[bar]` table of `config.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BarSection {
    /// The bar's name as written, so a value mosaico does not know is
    /// kept and reported rather than lost. See [`check`](Self::check).
    pub provider: String,
}

impl Default for BarSection {
    fn default() -> Self {
        Self {
            provider: BarChoice::Mosaico.name().to_string(),
        }
    }
}

impl BarSection {
    /// What the provider written stands for.
    pub fn check(&self) -> ProviderCheck {
        match BarChoice::parse(&self.provider) {
            Some(choice) => ProviderCheck::Known(choice),
            None => ProviderCheck::Unknown(self.provider.clone()),
        }
    }

    /// The bar in use: the one written, or `external` for a name mosaico
    /// does not know.
    pub fn choice(&self) -> BarChoice {
        self.check().choice()
    }

    /// The section of a `config.toml` whose `[bar]` table cannot be read:
    /// `external`, so the built-in bar never draws over a bar the user
    /// picked in a file mosaico cannot read.
    pub(crate) fn unreadable() -> Self {
        Self {
            provider: BarChoice::External.name().to_string(),
        }
    }
}

/// What `[bar] provider` stands for, and why when it is not what is
/// written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderCheck {
    /// A bar mosaico knows.
    Known(BarChoice),
    /// A name mosaico does not know, read as `external`.
    Unknown(String),
    /// `config.toml` does not load and its `[bar]` table cannot be read
    /// on its own either, so the bar is `external`.
    Unreadable,
}

impl ProviderCheck {
    /// The bar in use.
    pub fn choice(&self) -> BarChoice {
        match self {
            Self::Known(choice) => *choice,
            Self::Unknown(_) | Self::Unreadable => BarChoice::External,
        }
    }

    /// What to tell the user when the bar in use is not the one written,
    /// the same words wherever it is said.
    pub fn warning(&self) -> Option<String> {
        match self {
            Self::Known(_) => None,
            Self::Unknown(name) => Some(format!(
                "[bar] provider \"{name}\" is not a bar mosaico knows (mosaico, external or \
                 none), so it is read as \"external\" and the built-in bar stays off"
            )),
            Self::Unreadable => Some(
                "config.toml does not load and its [bar] table cannot be read on its own, so \
                 the provider is \"external\" and the built-in bar stays off until the file \
                 is fixed"
                    .to_string(),
            ),
        }
    }
}

/// The `[bar]` table of a `config.toml` holding `content`, read on its
/// own so that a mistake elsewhere in the file does not change the bar.
///
/// A file that is valid TOML gives its table, whatever the other tables
/// hold. One that is not gives the lines under `[bar]` and its
/// subtables, read alone. A file without the table has the default.
/// Returns `None` when the table is there but cannot be read.
pub(crate) fn read_bar_section(content: &str) -> Option<BarSection> {
    let table = match content.parse::<toml::Table>() {
        Ok(table) => table,
        Err(_) => bar_lines(content)?.parse::<toml::Table>().ok()?,
    };
    match table.get("bar") {
        Some(bar) => bar.clone().try_into().ok(),
        None => Some(BarSection::default()),
    }
}

/// The lines of `content` under the `[bar]` header and those of its
/// subtables, or an empty text when the file has no such header. `None`
/// when a top-level key names `bar` outside any header, which cannot be
/// read apart from the lines around it.
fn bar_lines(content: &str) -> Option<String> {
    let mut lines = String::new();
    let mut inside = false;
    let mut in_table = false;
    for line in content.lines() {
        if let Some(name) = header(line) {
            in_table = true;
            inside = name == "bar" || name.starts_with("bar.");
        } else if !in_table && names_bar(line) {
            return None;
        }
        if inside {
            lines.push_str(line);
            lines.push('\n');
        }
    }
    Some(lines)
}

/// The name in a `[table]` or `[[array]]` header, without spaces.
fn header(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix('[')?;
    let rest = rest.strip_prefix('[').unwrap_or(rest);
    let (name, _) = rest.split_once(']')?;
    Some(name.chars().filter(|c| !c.is_whitespace()).collect())
}

/// Whether `line`, outside any table, sets `bar` or one of its keys.
fn names_bar(line: &str) -> bool {
    let line = line.trim_start();
    line.strip_prefix("bar")
        .and_then(|rest| rest.trim_start().chars().next())
        .is_some_and(|next| next == '=' || next == '.')
}

#[cfg(test)]
#[path = "bar_section_tests.rs"]
mod tests;
