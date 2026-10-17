use super::*;
use crate::config::theme::CatppuccinFlavor;

#[test]
fn monitors_field_round_trips_through_toml() {
    let config = BarConfig {
        monitors: vec![0, 2],
        ..Default::default()
    };
    let toml_str = toml::to_string(&config).unwrap();
    let parsed: BarConfig = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.monitors, vec![0, 2]);
}

#[test]
fn monitors_field_from_toml() {
    let config: BarConfig = toml::from_str("monitors = [1]\n").unwrap();
    assert_eq!(config.monitors, vec![1]);
    assert!(!config.should_show_on(0));
    assert!(config.should_show_on(1));
}

#[test]
fn bar_config_round_trips_through_toml() {
    let mut config = BarConfig::default();
    config.resolve_colors(Theme::Catppuccin(CatppuccinFlavor::Mocha));
    let toml_str = toml::to_string(&config).unwrap();
    let parsed: BarConfig = toml::from_str(&toml_str).unwrap();

    assert_eq!(parsed.height, config.height);
    assert_eq!(parsed.font, config.font);
    assert_eq!(parsed.pill_radius, config.pill_radius);
    assert_eq!(parsed.left.len(), config.left.len());
    assert_eq!(parsed.right.len(), config.right.len());
}

#[test]
fn empty_toml_uses_all_defaults() {
    let config: BarConfig = toml::from_str("").unwrap();

    assert!(config.enabled);
    assert_eq!(config.height, 64);
    assert_eq!(config.font, "CaskaydiaCove Nerd Font");
    assert_eq!(config.left.len(), 3);
}

#[test]
fn partial_toml_preserves_unset_defaults() {
    let config: BarConfig = toml::from_str("height = 40\nfont = \"Hack\"\n").unwrap();

    assert_eq!(config.height, 40);
    assert_eq!(config.font, "Hack");
    assert_eq!(config.font_size, 24);
}

#[test]
fn widget_with_icon_deserializes() {
    let toml_str = "[[left]]\ntype = \"layout\"\nicon = \"X\"\n";
    let config: BarConfig = toml::from_str(toml_str).unwrap();

    assert_eq!(config.left.len(), 1);
    assert_eq!(config.left[0].icon(), "X");
}

#[test]
fn widget_without_icon_defaults_to_empty() {
    let toml_str = "[[left]]\ntype = \"layout\"\n";
    let config: BarConfig = toml::from_str(toml_str).unwrap();

    assert_eq!(config.left[0].icon(), "");
}

#[test]
fn clock_with_icon_and_format() {
    let toml_str = r#"
        [[right]]
        type = "clock"
        format = "%H:%M"
        icon = "C"
    "#;
    let config: BarConfig = toml::from_str(toml_str).unwrap();

    match &config.right[0] {
        WidgetConfig::Clock { format, icon, .. } => {
            assert_eq!(format, "%H:%M");
            assert_eq!(icon, "C");
        }
        other => panic!("expected Clock, got {other:?}"),
    }
}
