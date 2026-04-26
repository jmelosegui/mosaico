use super::*;
use crate::layout::LayoutKind;

#[test]
fn default_config_has_expected_values() {
    let mut config = Config::default();
    config.validate();

    assert_eq!(
        config.theme.resolve(),
        Theme::Catppuccin(CatppuccinFlavor::Mocha)
    );
    assert_eq!(config.layout.gap, 8);
    assert_eq!(config.borders.width, 4);
}

#[test]
fn validate_resolves_border_colors_from_theme() {
    let mut config = Config::default();
    config.validate();

    // Mocha defaults: Blue for focused, Green for monocle
    assert_eq!(config.borders.colors.focused, "#89b4fa");
    assert_eq!(config.borders.colors.monocle, "#a6e3a1");
}

#[test]
fn explicit_border_color_overrides_theme() {
    let mut config = Config {
        borders: BorderConfig {
            colors: BorderColors {
                focused: "#ff0000".into(),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    config.validate();

    assert_eq!(config.borders.colors.focused, "#ff0000");
    assert_eq!(config.borders.colors.monocle, "#a6e3a1"); // still from theme
}

#[test]
fn latte_theme_resolves_different_borders() {
    let mut config = Config {
        theme: ThemeConfig {
            name: "catppuccin".into(),
            flavor: "latte".into(),
        },
        ..Default::default()
    };
    config.validate();

    assert_eq!(config.borders.colors.focused, "#1e66f5");
    assert_eq!(config.borders.colors.monocle, "#40a02b");
}

#[test]
fn named_color_in_border_resolves_to_hex() {
    let mut config = Config {
        borders: BorderConfig {
            colors: BorderColors {
                focused: "mauve".into(),
                monocle: "teal".into(),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    config.validate();

    assert_eq!(config.borders.colors.focused, "#cba6f7");
    assert_eq!(config.borders.colors.monocle, "#94e2d5");
}

#[test]
fn default_keybindings_are_not_empty() {
    // Act
    let bindings = keybinding::defaults();

    // Assert
    assert!(!bindings.is_empty());
}

#[test]
fn partial_toml_uses_defaults_for_missing_sections() {
    // Arrange
    let toml_str = "[layout]\ngap = 16\n";

    // Act
    let config: Config = toml::from_str(toml_str).unwrap();

    // Assert
    assert_eq!(config.layout.gap, 16);
    assert_eq!(config.layout.ratio, 0.5);
}

#[test]
fn workspaces_layouts_parses_string_keys_as_u8() {
    // Arrange
    let toml_str = r#"
[workspaces.layouts]
1 = "vertical-stack"
3 = "three-column"
"#;

    // Act
    let config: Config = toml::from_str(toml_str).unwrap();

    // Assert
    assert_eq!(
        config.workspaces.layouts.get(&1).copied(),
        Some(LayoutKind::VerticalStack)
    );
    assert_eq!(
        config.workspaces.layouts.get(&3).copied(),
        Some(LayoutKind::ThreeColumn)
    );
    assert!(!config.workspaces.layouts.contains_key(&2));
}

#[test]
fn workspaces_layouts_drops_out_of_range_keys() {
    // Arrange: 0 and 9 are out of the valid 1..=8 range; 2 is valid.
    let toml_str = r#"
[workspaces.layouts]
0 = "bsp"
2 = "vertical-stack"
9 = "three-column"
"#;

    // Act
    let config: Config = toml::from_str(toml_str).unwrap();

    // Assert
    assert!(!config.workspaces.layouts.contains_key(&0));
    assert!(!config.workspaces.layouts.contains_key(&9));
    assert_eq!(
        config.workspaces.layouts.get(&2).copied(),
        Some(LayoutKind::VerticalStack)
    );
}

#[test]
fn workspaces_layouts_drops_non_numeric_keys() {
    // Arrange: a non-numeric key should be skipped without failing the
    // whole parse.
    let toml_str = r#"
[workspaces.layouts]
abc = "bsp"
1 = "three-column"
"#;

    // Act
    let config: Config = toml::from_str(toml_str).unwrap();

    // Assert
    assert_eq!(config.workspaces.layouts.len(), 1);
    assert_eq!(
        config.workspaces.layouts.get(&1).copied(),
        Some(LayoutKind::ThreeColumn)
    );
}

#[test]
fn unfocused_border_defaults_to_theme_when_unset() {
    // A bare config with no [borders.colors] section should resolve
    // unfocused to the Mocha theme's muted gray (Overlay0).
    let mut config = Config::default();
    config.validate();
    assert_eq!(config.borders.colors.unfocused, "#6c7086");
    assert!(config.borders.colors.unfocused_enabled());
}

#[test]
fn unfocused_none_sentinel_disables_unfocused_borders() {
    // The literal value "none" is preserved verbatim through validation
    // and reported as disabled.
    let toml_str = r#"
[borders.colors]
unfocused = "none"
"#;
    let mut config: Config = toml::from_str(toml_str).unwrap();
    config.validate();
    assert_eq!(config.borders.colors.unfocused, "none");
    assert!(!config.borders.colors.unfocused_enabled());
}

#[test]
fn explicit_unfocused_hex_passes_through_unchanged() {
    let toml_str = r##"
[borders.colors]
unfocused = "#deadbe"
"##;
    let mut config: Config = toml::from_str(toml_str).unwrap();
    config.validate();
    assert_eq!(config.borders.colors.unfocused, "#deadbe");
    assert!(config.borders.colors.unfocused_enabled());
}

#[test]
fn named_unfocused_color_resolves_via_theme_palette() {
    // Named accent colors are resolved through the active theme.
    // Mocha's mauve is a known Catppuccin hex.
    let toml_str = r#"
[borders.colors]
unfocused = "mauve"
"#;
    let mut config: Config = toml::from_str(toml_str).unwrap();
    config.validate();
    // Mocha mauve = #cba6f7
    assert_eq!(config.borders.colors.unfocused, "#cba6f7");
    assert!(config.borders.colors.unfocused_enabled());
}

#[test]
fn workspaces_layouts_missing_section_is_empty_default() {
    // Arrange: no [workspaces.layouts] section at all.
    let toml_str = "[layout]\ngap = 8\n";

    // Act
    let config: Config = toml::from_str(toml_str).unwrap();

    // Assert
    assert!(config.workspaces.layouts.is_empty());
}

#[test]
fn rule_excludes_by_class() {
    // Arrange
    let rules = vec![WindowRule {
        match_class: Some("TaskManager".into()),
        match_title: None,
        manage: false,
    }];

    // Act / Assert
    assert!(!should_manage("TaskManager", "Task Manager", &rules));
    assert!(should_manage("Notepad", "Untitled", &rules));
}

#[test]
fn rule_excludes_by_title_substring() {
    // Arrange
    let rules = vec![WindowRule {
        match_class: None,
        match_title: Some("settings".into()),
        manage: false,
    }];

    // Act / Assert
    assert!(!should_manage("App", "Windows Settings", &rules));
    assert!(should_manage("App", "My Document", &rules));
}

#[test]
fn first_matching_rule_wins() {
    // Arrange
    let rules = vec![
        WindowRule {
            match_class: Some("Chrome".into()),
            match_title: None,
            manage: false,
        },
        WindowRule {
            match_class: Some("Chrome".into()),
            match_title: None,
            manage: true,
        },
    ];

    // Act / Assert
    assert!(!should_manage("Chrome", "Google", &rules));
}

#[test]
fn no_rules_defaults_to_manage() {
    // Act / Assert
    assert!(should_manage("Any", "Window", &[]));
}

#[test]
fn validate_clamps_extreme_values() {
    // Arrange
    let mut config = Config {
        layout: LayoutConfig {
            gap: -50,
            ratio: 2.0,
            ..Default::default()
        },
        borders: BorderConfig {
            width: 999,
            ..Default::default()
        },
        ..Default::default()
    };

    // Act
    config.validate();

    // Assert
    assert_eq!(config.layout.gap, 0);
    assert!((config.layout.ratio - 0.9).abs() < f64::EPSILON);
    assert_eq!(config.borders.width, 32);
}

#[test]
fn empty_match_title_only_matches_empty_title() {
    // Arrange
    let rules = vec![WindowRule {
        match_class: Some("ApplicationFrameWindow".into()),
        match_title: Some(String::new()),
        manage: false,
    }];

    // Act / Assert
    assert!(!should_manage("ApplicationFrameWindow", "", &rules));
    assert!(should_manage("ApplicationFrameWindow", "Settings", &rules));
}

#[test]
fn default_rules_are_empty() {
    // Arrange / Act
    let rules = default_rules();

    // Assert
    assert!(rules.is_empty());
}
