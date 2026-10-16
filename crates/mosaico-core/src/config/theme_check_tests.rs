//! Whether a theme name and flavor are known, which `mosaico doctor`
//! reports, against the themes the daemon draws with.

use crate::config::Config;

use super::*;

fn config(name: &str, flavor: &str) -> ThemeConfig {
    ThemeConfig {
        name: name.into(),
        flavor: flavor.into(),
    }
}

/// The cases whose check does not give the answer expected, described.
fn mismatches(cases: &[(&str, &str, ThemeCheck)]) -> Vec<String> {
    cases
        .iter()
        .filter_map(|&(name, flavor, expected)| {
            let got = config(name, flavor).check();
            (got != expected)
                .then(|| format!("{name:?} {flavor:?}: {got:?}, expected {expected:?}"))
        })
        .collect()
}

/// Every theme mosaico has, one per family and flavor.
const THEMES: [Theme; 10] = [
    Theme::Catppuccin(CatppuccinFlavor::Mocha),
    Theme::Catppuccin(CatppuccinFlavor::Macchiato),
    Theme::Catppuccin(CatppuccinFlavor::Frappe),
    Theme::Catppuccin(CatppuccinFlavor::Latte),
    Theme::RosePine(RosePineFlavor::Main),
    Theme::RosePine(RosePineFlavor::Moon),
    Theme::RosePine(RosePineFlavor::Dawn),
    Theme::TokyoNight(TokyoNightFlavor::Night),
    Theme::TokyoNight(TokyoNightFlavor::Storm),
    Theme::TokyoNight(TokyoNightFlavor::Day),
];

#[test]
fn every_spelling_and_flavor_of_rose_pine_and_tokyo_night_is_known() {
    // Arrange -- #115: doctor called these unknown themes, though the
    // daemon draws with them.
    let rose_pine = [
        ("main", RosePineFlavor::Main),
        ("moon", RosePineFlavor::Moon),
        ("dawn", RosePineFlavor::Dawn),
        ("Dawn", RosePineFlavor::Dawn),
    ];
    let tokyo_night = [
        ("night", TokyoNightFlavor::Night),
        ("storm", TokyoNightFlavor::Storm),
        ("day", TokyoNightFlavor::Day),
        ("STORM", TokyoNightFlavor::Storm),
    ];
    let mut cases = Vec::new();
    for name in ["rose-pine", "rosepine", "rose_pine", "Rose-Pine"] {
        for (flavor, known) in rose_pine {
            cases.push((name, flavor, ThemeCheck::Known(Theme::RosePine(known))));
        }
    }
    for name in ["tokyo-night", "tokyonight", "tokyo_night", "TokyoNight"] {
        for (flavor, known) in tokyo_night {
            cases.push((name, flavor, ThemeCheck::Known(Theme::TokyoNight(known))));
        }
    }

    // Act
    let wrong = mismatches(&cases);

    // Assert
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn an_unknown_flavor_of_rose_pine_or_tokyo_night_falls_back_to_its_own() {
    // Arrange -- #115: doctor called the theme unknown instead.
    let cases = [
        (
            "rose-pine",
            "mocha",
            ThemeCheck::UnknownFlavor(Theme::RosePine(RosePineFlavor::Main)),
        ),
        (
            "tokyo-night",
            "latte",
            ThemeCheck::UnknownFlavor(Theme::TokyoNight(TokyoNightFlavor::Night)),
        ),
    ];

    // Act
    let wrong = mismatches(&cases);

    // Assert
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn every_theme_is_known_by_the_name_and_flavor_it_is_spelled_with() {
    // Arrange -- #115: the names doctor prints must be ones the check,
    // and the daemon, take back.
    let cases: Vec<_> = THEMES
        .iter()
        .map(|&theme| (theme.name(), theme.flavor_name(), ThemeCheck::Known(theme)))
        .collect();

    // Act
    let wrong = mismatches(&cases);

    // Assert
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn a_theme_named_without_a_flavor_has_the_default_flavor_of_its_family() {
    // Arrange -- #115: the user guide calls Main and Night the family
    // defaults, so leaving the flavor out is a way to pick them.
    let cases = [
        ("catppuccin", Theme::Catppuccin(CatppuccinFlavor::Mocha)),
        ("rose-pine", Theme::RosePine(RosePineFlavor::Main)),
        ("tokyo-night", Theme::TokyoNight(TokyoNightFlavor::Night)),
    ];

    // Act
    let wrong: Vec<_> = cases
        .iter()
        .filter_map(|&(name, expected)| {
            let file = format!("[theme]\nname = {name:?}\n");
            let config: Config = toml::from_str(&file).unwrap();
            let got = config.theme.check();
            (got != ThemeCheck::Known(expected)).then(|| format!("{name:?}: {got:?}"))
        })
        .collect();

    // Assert
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn catppuccin_and_its_flavors_are_checked_as_before() {
    // Arrange -- the near miss: what doctor has always known.
    let catppuccin = |flavor| ThemeCheck::Known(Theme::Catppuccin(flavor));
    let cases = [
        ("catppuccin", "mocha", catppuccin(CatppuccinFlavor::Mocha)),
        (
            "catppuccin",
            "macchiato",
            catppuccin(CatppuccinFlavor::Macchiato),
        ),
        ("catppuccin", "frappe", catppuccin(CatppuccinFlavor::Frappe)),
        ("catppuccin", "frappé", catppuccin(CatppuccinFlavor::Frappe)),
        ("Catppuccin", "LATTE", catppuccin(CatppuccinFlavor::Latte)),
        (
            "catppuccin",
            "espresso",
            ThemeCheck::UnknownFlavor(Theme::Catppuccin(CatppuccinFlavor::Mocha)),
        ),
    ];

    // Act
    let wrong = mismatches(&cases);

    // Assert
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn a_name_that_is_no_theme_is_unknown_and_falls_back_to_the_default() {
    // Arrange -- the near miss: really unknown names stay unknown.
    let unknown = ThemeCheck::UnknownName(Theme::default());
    let cases = [
        ("dracula", "night", unknown),
        ("", "mocha", unknown),
        ("rose pine", "moon", unknown),
    ];

    // Act
    let wrong = mismatches(&cases);

    // Assert
    assert!(wrong.is_empty(), "{wrong:#?}");
}
