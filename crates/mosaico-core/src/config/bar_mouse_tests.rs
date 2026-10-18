use crate::config::BarConfig;

use super::*;

#[test]
fn the_bar_takes_the_mouse_and_scrolls_through_workspaces_by_default() {
    // Act
    let config: BarConfig = toml::from_str("").unwrap();

    // Assert
    assert!(config.mouse.enabled);
    assert_eq!(config.mouse.scroll, BarScroll::Workspaces);
    assert_eq!(config.mouse, BarMouseConfig::default());
}

#[test]
fn the_mouse_section_turns_the_bar_click_through_and_the_wheel_off() {
    // Act
    let config: BarConfig =
        toml::from_str("[mouse]\nenabled = false\nscroll = \"none\"\n").unwrap();

    // Assert
    assert!(!config.mouse.enabled);
    assert_eq!(config.mouse.scroll, BarScroll::None);
}

#[test]
fn a_section_with_one_key_keeps_the_default_of_the_other() {
    // Act
    let config: BarConfig = toml::from_str("[mouse]\nscroll = \"none\"\n").unwrap();

    // Assert
    assert!(config.mouse.enabled);
}

#[test]
fn an_unknown_scroll_makes_the_file_invalid() {
    // Act
    let result: Result<BarConfig, _> = toml::from_str("[mouse]\nscroll = \"layouts\"\n");

    // Assert -- as any other unknown value, so the daemon says why.
    assert!(result.is_err());
}

#[test]
fn the_mouse_section_round_trips_through_toml() {
    // Arrange
    let mut config = BarConfig::default();
    config.mouse.scroll = BarScroll::None;

    // Act
    let text = toml::to_string(&config).unwrap();
    let parsed: BarConfig = toml::from_str(&text).unwrap();

    // Assert
    assert_eq!(parsed.mouse, config.mouse);
}
