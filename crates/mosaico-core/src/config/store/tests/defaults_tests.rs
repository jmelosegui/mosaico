//! What the loads fall back to when a file is missing or does not parse.

use crate::config::store::test_support::temp_store;
use crate::config::{CatppuccinFlavor, Theme};

use super::*;

/// The colors the borders of `config` are drawn in: focused, monocle and
/// unfocused.
fn borders(config: &Config) -> [&str; 3] {
    let colors = &config.borders.colors;
    [&colors.focused, &colors.monocle, &colors.unfocused]
}

/// The border colors of `theme`, in the same order.
fn theme_borders(theme: Theme) -> [&'static str; 3] {
    [
        theme.border_focused(),
        theme.border_monocle(),
        theme.border_unfocused(),
    ]
}

#[test]
fn without_a_config_file_the_borders_take_the_theme_colors() {
    // Arrange -- #101: an install that never ran `mosaico init`.
    let store = temp_store("defaults-missing");

    // Act
    let config = store.load_config();

    // Assert
    assert_eq!(borders(&config), theme_borders(config.theme.resolve()));
}

#[test]
fn with_a_config_file_that_does_not_parse_the_borders_take_the_theme_colors() {
    // Arrange -- #101: the warning is printed and the defaults are used.
    let store = temp_store("defaults-invalid");
    std::fs::write(store.config_path(), "not = [valid").unwrap();

    // Act
    let config = store.load_config();

    // Assert
    assert_eq!(borders(&config), theme_borders(config.theme.resolve()));
}

#[test]
fn a_config_file_without_borders_takes_the_colors_of_its_own_theme() {
    // Arrange -- the near miss: a file that parses has always been
    // validated, and must keep the colors of the theme it names.
    let store = temp_store("defaults-latte");
    let file = "[theme]\nname = \"catppuccin\"\nflavor = \"latte\"\n\n[layout]\ngap = 12\n";
    std::fs::write(store.config_path(), file).unwrap();

    // Act
    let config = store.load_config();

    // Assert
    let latte = Theme::Catppuccin(CatppuccinFlavor::Latte);
    assert_eq!(config.layout.gap, 12);
    assert_eq!(borders(&config), theme_borders(latte));
}

#[test]
fn without_a_bar_file_the_defaults_are_what_validation_would_give() {
    // Arrange -- the bar falls back to its defaults without validating
    // them too, which is right only while every default is within the
    // limits validation enforces. Its colors are filled from the theme
    // where the bar is drawn, for a loaded bar and the defaults alike.
    let store = temp_store("defaults-bar");
    let mut validated = BarConfig::default();
    validated.validate();

    // Act
    let bar = store.load_bar();

    // Assert
    assert_eq!(format!("{bar:?}"), format!("{validated:?}"));
}
