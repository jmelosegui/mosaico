use super::*;
use crate::config::rules;

#[test]
fn config_template_parses_as_valid_config() {
    // Arrange
    let toml_str = generate_config();

    // Act
    let result: Result<crate::Config, _> = toml::from_str(&toml_str);

    // Assert
    assert!(
        result.is_ok(),
        "config template is not valid TOML: {result:?}"
    );
}

#[test]
fn config_template_matches_default_values() {
    // Arrange
    let toml_str = generate_config();

    // Act
    let mut config: crate::Config = toml::from_str(&toml_str).unwrap();
    config.validate();

    // Assert
    let mut defaults = crate::Config::default();
    defaults.validate();
    assert_eq!(config.theme, defaults.theme);
    assert_eq!(config.theme.name, "catppuccin");
    assert_eq!(config.theme.flavor, "mocha");
    assert_eq!(config.layout.gap, defaults.layout.gap);
    assert_eq!(config.layout.ratio, defaults.layout.ratio);
    assert_eq!(config.borders.width, defaults.borders.width);
    assert_eq!(config.mouse.follows_focus, defaults.mouse.follows_focus);
    assert_eq!(
        config.mouse.focus_follows_mouse,
        defaults.mouse.focus_follows_mouse
    );
    // Border colors should resolve from the default Mocha theme.
    assert_eq!(config.borders.focused, defaults.borders.focused);
    assert_eq!(config.borders.monocle, defaults.borders.monocle);
}

#[test]
fn keybindings_template_parses_correctly() {
    // Arrange
    let toml_str = generate_keybindings();

    // Act
    let result: Result<rules::KeybindingsFile, _> = toml::from_str(&toml_str);

    // Assert
    assert!(
        result.is_ok(),
        "keybindings template is not valid TOML: {result:?}"
    );
}

#[test]
fn keybindings_template_matches_defaults() {
    // Arrange
    let toml_str = generate_keybindings();

    // Act
    let file: rules::KeybindingsFile = toml::from_str(&toml_str).unwrap();

    // Assert
    let defaults = crate::config::keybinding::defaults();
    assert_eq!(file.keybinding.len(), defaults.len());
}

#[test]
fn rules_template_parses_correctly() {
    // Arrange
    let toml_str = generate_rules();

    // Act
    let result: Result<rules::RulesFile, _> = toml::from_str(&toml_str);

    // Assert
    assert!(
        result.is_ok(),
        "rules template is not valid TOML: {result:?}"
    );
}

#[test]
fn rules_template_has_rules() {
    // Arrange
    let toml_str = generate_rules();

    // Act
    let file: rules::RulesFile = toml::from_str(&toml_str).unwrap();

    // Assert — template contains community rules from the repo.
    assert!(!file.rule.is_empty());
}

#[test]
fn user_rules_template_parses_with_zero_rules() {
    // Arrange
    let toml_str = generate_user_rules();

    // Act
    let file: rules::UserRulesFile = toml::from_str(&toml_str).unwrap();

    // Assert — all entries are commented out, so zero rules.
    assert_eq!(file.rule.len(), 0);
}

#[test]
fn bar_template_parses_as_valid_bar_config() {
    // Arrange
    let toml_str = generate_bar();

    // Act
    let result: Result<crate::BarConfig, _> = toml::from_str(&toml_str);

    // Assert
    assert!(result.is_ok(), "bar template is not valid TOML: {result:?}");
}

#[test]
fn bar_template_matches_default_values() {
    // Arrange
    let toml_str = generate_bar();

    // Act
    let mut config: crate::BarConfig = toml::from_str(&toml_str).unwrap();
    config.validate();

    // Assert
    let mut defaults = crate::BarConfig::default();
    defaults.validate();
    assert_eq!(config.height, defaults.height);
    assert_eq!(config.font, defaults.font);
    assert_eq!(config.font_size, defaults.font_size);
    assert_eq!(config.padding, defaults.padding);
    assert_eq!(config.pill_padding, defaults.pill_padding);
    assert_eq!(config.pill_radius, defaults.pill_radius);
    assert_eq!(config.item_gap, defaults.item_gap);
    assert_eq!(config.separator, defaults.separator);
    assert_eq!(config.background_opacity, defaults.background_opacity);
    assert_eq!(config.monitors, defaults.monitors);
    assert_eq!(config.left.len(), defaults.left.len());
    assert_eq!(config.right.len(), defaults.right.len());
}
