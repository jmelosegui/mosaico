use super::*;
use crate::config::theme::CatppuccinFlavor;

#[test]
fn resolve_colors_fills_from_theme() {
    let mut config = BarConfig::default();
    config.resolve_colors(Theme::Catppuccin(CatppuccinFlavor::Mocha));

    assert_eq!(config.colors.background, "#1e1e2e");
    assert_eq!(config.colors.widget_background, "#313244");
    assert_eq!(config.colors.active_workspace_text, "#cdd6f4");
}

#[test]
fn resolve_colors_latte_fills_light_colors() {
    let mut config = BarConfig::default();
    config.resolve_colors(Theme::Catppuccin(CatppuccinFlavor::Latte));
    assert_eq!(config.colors.background, "#eff1f5");
    assert_eq!(config.colors.foreground, "#1e66f5");
}

#[test]
fn explicit_color_overrides_theme() {
    let toml_str = "[colors]\nbackground = \"#000000\"\n";
    let mut config: BarConfig = toml::from_str(toml_str).unwrap();
    config.resolve_colors(Theme::Catppuccin(CatppuccinFlavor::Latte));

    // Explicit override kept
    assert_eq!(config.colors.background, "#000000");
    // Unset fields resolved from latte
    assert_eq!(config.colors.foreground, "#1e66f5");
}

#[test]
fn named_color_in_bar_resolves_to_hex() {
    let toml_str = "[colors]\naccent = \"mauve\"\n";
    let mut config: BarConfig = toml::from_str(toml_str).unwrap();
    config.resolve_colors(Theme::Catppuccin(CatppuccinFlavor::Mocha));

    assert_eq!(config.colors.accent, "#cba6f7");
    // Unset fields still resolved from theme
    assert_eq!(config.colors.background, "#1e1e2e");
}

#[test]
fn widget_color_resolves_named_to_hex() {
    let mut config = BarConfig::default();
    config.resolve_colors(Theme::Catppuccin(CatppuccinFlavor::Mocha));
    // Update widget defaults to "green", resolved to Mocha green hex.
    let update = config
        .right
        .iter()
        .find(|w| matches!(w, WidgetConfig::Update { .. }))
        .unwrap();
    assert_eq!(update.color(), "#a6e3a1");
}

#[test]
fn widget_color_empty_means_no_override() {
    let mut config = BarConfig::default();
    config.resolve_colors(Theme::Catppuccin(CatppuccinFlavor::Mocha));
    // Clock has no custom color -- stays empty.
    let clock = config
        .right
        .iter()
        .find(|w| matches!(w, WidgetConfig::Clock { .. }))
        .unwrap();
    assert!(clock.color().is_empty());
}

#[test]
fn widget_color_from_toml() {
    let toml_str = "[[left]]\ntype = \"layout\"\ncolor = \"red\"\n";
    let mut config: BarConfig = toml::from_str(toml_str).unwrap();
    config.resolve_colors(Theme::Catppuccin(CatppuccinFlavor::Mocha));
    assert_eq!(config.left[0].color(), "#f38ba8");
}
