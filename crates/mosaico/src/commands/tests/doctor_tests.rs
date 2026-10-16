use mosaico_core::config::ThemeConfig;

use super::*;
use crate::commands::test_support::Harness;

fn theme(name: &str, flavor: &str) -> ThemeConfig {
    ThemeConfig {
        name: name.into(),
        flavor: flavor.into(),
    }
}

/// The `[theme]` doctor reads from a `config.toml` holding `contents`.
fn loaded(test: &str, contents: &str) -> ThemeConfig {
    let h = Harness::new(test);
    std::fs::create_dir_all(h.config.dir()).unwrap();
    std::fs::write(h.config.config_path(), contents).unwrap();
    let theme = h.config.load_config().theme;
    let _ = std::fs::remove_dir_all(h.config.dir());
    theme
}

#[test]
fn rose_pine_and_tokyo_night_are_reported_as_the_theme_in_use() {
    // Arrange -- #115: doctor warned "Unknown theme", falling back to
    // Catppuccin Mocha, while the daemon draws with these.
    let rose_pine = theme("rose_pine", "moon");
    let tokyo_night = theme("TokyoNight", "Storm");

    // Act
    let lines = [theme_line(&rose_pine), theme_line(&tokyo_night)];

    // Assert
    assert_eq!(
        lines,
        [
            format!("  {OK} Theme: rose-pine moon"),
            format!("  {OK} Theme: tokyo-night storm"),
        ]
    );
}

#[test]
fn an_unknown_flavor_of_a_known_theme_names_the_flavor_it_falls_back_to() {
    // Arrange -- #115: doctor called the whole theme unknown instead.
    let tokyo_night = theme("tokyo-night", "latte");

    // Act
    let line = theme_line(&tokyo_night);

    // Assert
    assert_eq!(
        line,
        format!("  {WARN} Unknown flavor \"latte\", falling back to night")
    );
}

#[test]
fn a_theme_named_without_a_flavor_is_reported_with_its_default_flavor() {
    // Arrange -- #115, as reported: `[theme]` with only the name.
    let tokyo_night = loaded("doctor-tokyo-night", "[theme]\nname = \"tokyo-night\"\n");
    let rose_pine = loaded("doctor-rose-pine", "[theme]\nname = \"rose-pine\"\n");

    // Act
    let lines = [theme_line(&tokyo_night), theme_line(&rose_pine)];

    // Assert
    assert_eq!(
        lines,
        [
            format!("  {OK} Theme: tokyo-night night"),
            format!("  {OK} Theme: rose-pine main"),
        ]
    );
}

#[test]
fn catppuccin_is_reported_as_before() {
    // Arrange -- the near miss: the lines doctor has always printed.
    let cases = [
        (
            theme("catppuccin", "mocha"),
            format!("  {OK} Theme: catppuccin mocha"),
        ),
        (
            theme("catppuccin", "frappé"),
            format!("  {OK} Theme: catppuccin frappé"),
        ),
        (
            theme("Catppuccin", "LATTE"),
            format!("  {OK} Theme: catppuccin latte"),
        ),
        (
            theme("catppuccin", "Espresso"),
            format!("  {WARN} Unknown flavor \"Espresso\", falling back to mocha"),
        ),
        (
            loaded("doctor-catppuccin", "[theme]\nname = \"catppuccin\"\n"),
            format!("  {OK} Theme: catppuccin mocha"),
        ),
    ];

    // Act / Assert
    for (config, expected) in cases {
        assert_eq!(theme_line(&config), expected, "{config:?}");
    }
}

#[test]
fn a_name_that_is_no_theme_is_reported_as_before() {
    // Arrange -- the near miss: a really unknown name.
    let dracula = theme("Dracula", "night");

    // Act
    let line = theme_line(&dracula);

    // Assert
    assert_eq!(
        line,
        format!("  {WARN} Unknown theme \"Dracula\", falling back to catppuccin mocha")
    );
}
