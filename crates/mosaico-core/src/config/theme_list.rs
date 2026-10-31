//! Every theme mosaico ships, and the names people read for them.

use super::{CatppuccinFlavor, RosePineFlavor, Theme, TokyoNightFlavor};

impl Theme {
    /// Every theme, family by family, each family's flavors in the order
    /// their authors list them.
    pub const ALL: [Theme; 10] = [
        Self::Catppuccin(CatppuccinFlavor::Mocha),
        Self::Catppuccin(CatppuccinFlavor::Macchiato),
        Self::Catppuccin(CatppuccinFlavor::Frappe),
        Self::Catppuccin(CatppuccinFlavor::Latte),
        Self::RosePine(RosePineFlavor::Main),
        Self::RosePine(RosePineFlavor::Moon),
        Self::RosePine(RosePineFlavor::Dawn),
        Self::TokyoNight(TokyoNightFlavor::Night),
        Self::TokyoNight(TokyoNightFlavor::Storm),
        Self::TokyoNight(TokyoNightFlavor::Day),
    ];

    /// The theme's family as people write it, such as `"Rosé Pine"`.
    pub fn label(self) -> &'static str {
        match self {
            Self::Catppuccin(_) => "Catppuccin",
            Self::RosePine(_) => "Rosé Pine",
            Self::TokyoNight(_) => "Tokyo Night",
        }
    }

    /// The theme's flavor as people write it, such as `"Frappé"`.
    pub fn flavor_label(self) -> &'static str {
        match self {
            Self::Catppuccin(f) => match f {
                CatppuccinFlavor::Mocha => "Mocha",
                CatppuccinFlavor::Macchiato => "Macchiato",
                CatppuccinFlavor::Frappe => "Frappé",
                CatppuccinFlavor::Latte => "Latte",
            },
            Self::RosePine(f) => match f {
                RosePineFlavor::Main => "Main",
                RosePineFlavor::Moon => "Moon",
                RosePineFlavor::Dawn => "Dawn",
            },
            Self::TokyoNight(f) => match f {
                TokyoNightFlavor::Night => "Night",
                TokyoNightFlavor::Storm => "Storm",
                TokyoNightFlavor::Day => "Day",
            },
        }
    }

    /// The flavors of this theme's family, in [`Theme::ALL`] order.
    pub fn flavors(self) -> impl Iterator<Item = Theme> {
        Self::ALL
            .into_iter()
            .filter(move |t| t.name() == self.name())
    }

    /// The first flavor of each family, in [`Theme::ALL`] order.
    pub fn families() -> impl Iterator<Item = Theme> {
        Self::ALL
            .into_iter()
            .enumerate()
            .filter(|&(i, t)| i == 0 || Self::ALL[i - 1].name() != t.name())
            .map(|(_, t)| t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ThemeConfig;

    #[test]
    fn every_theme_reads_back_from_its_config_names() {
        for theme in Theme::ALL {
            // Arrange
            let config = ThemeConfig {
                name: theme.name().into(),
                flavor: theme.flavor_name().into(),
            };

            // Act / Assert
            assert_eq!(config.resolve(), theme);
        }
    }

    #[test]
    fn every_flavor_has_its_own_label() {
        let labels: std::collections::HashSet<_> =
            Theme::ALL.iter().map(|t| t.flavor_label()).collect();
        assert_eq!(labels.len(), Theme::ALL.len());
    }

    #[test]
    fn families_lists_each_family_once_with_its_flavors() {
        let families: Vec<_> = Theme::families().map(Theme::label).collect();
        assert_eq!(families, ["Catppuccin", "Rosé Pine", "Tokyo Night"]);
        let rose_pine: Vec<_> = Theme::RosePine(RosePineFlavor::Moon)
            .flavors()
            .map(Theme::flavor_label)
            .collect();
        assert_eq!(rose_pine, ["Main", "Moon", "Dawn"]);
    }
}
