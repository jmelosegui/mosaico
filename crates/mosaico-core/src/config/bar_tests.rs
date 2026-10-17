use super::*;

#[test]
fn default_bar_config_has_expected_values() {
    let config = BarConfig::default();

    assert!(config.enabled);
    assert_eq!(config.height, 64);
    assert_eq!(config.font, "CaskaydiaCove Nerd Font");
    assert_eq!(config.font_size, 24);
    assert_eq!(config.pill_radius, 4);
    assert_eq!(config.pill_border_width, 2);
    assert!(config.font_bold);
    assert!(!config.font_italic);
    assert!(!config.font_underline);
    assert_eq!(config.background_opacity, 0);
    assert!(config.monitors.is_empty());
    assert_eq!(config.left.len(), 3);
    assert_eq!(config.right.len(), 6);
}

#[test]
fn should_show_on_empty_monitors_means_all() {
    let config = BarConfig::default();
    assert!(config.should_show_on(0));
    assert!(config.should_show_on(1));
    assert!(config.should_show_on(99));
}

#[test]
fn should_show_on_filters_by_index() {
    let config = BarConfig {
        monitors: vec![0, 2],
        ..Default::default()
    };
    assert!(config.should_show_on(0));
    assert!(!config.should_show_on(1));
    assert!(config.should_show_on(2));
}

#[test]
fn should_show_on_respects_enabled() {
    let config = BarConfig {
        enabled: false,
        ..Default::default()
    };
    assert!(!config.should_show_on(0));
}

#[test]
fn validate_clamps_extreme_values() {
    let mut config = BarConfig {
        height: 200,
        font_size: 0,
        padding: -5,
        pill_radius: 100,
        background_opacity: 150,
        ..Default::default()
    };

    config.validate();

    assert_eq!(config.height, 96);
    assert_eq!(config.font_size, 8);
    assert_eq!(config.padding, 0);
    assert_eq!(config.pill_radius, 32);
    assert_eq!(config.background_opacity, 100);
}

#[test]
fn validate_preserves_valid_values() {
    let mut config = BarConfig::default();
    config.validate();
    assert_eq!(config.height, 64);
    assert_eq!(config.pill_radius, 4);
}
