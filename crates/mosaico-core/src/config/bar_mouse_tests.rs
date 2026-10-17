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

#[test]
fn the_clicks_are_bound_on_the_layout_and_paused_widgets_by_default() {
    // Act
    let config: BarConfig = toml::from_str("[mouse]\nenabled = true\n").unwrap();

    // Assert
    assert_eq!(
        config.mouse.click,
        [
            ClickBinding::new("layout", "left", "cycle-layout"),
            ClickBinding::new("layout", "right", "toggle-monocle"),
            ClickBinding::new("paused", "left", "toggle-pause"),
        ]
    );
}

#[test]
fn the_click_entries_are_read_as_the_file_writes_them() {
    // Act
    let config: BarConfig = toml::from_str(
        r#"
        [[mouse.click]]
        widget = "clock"
        button = "middle"
        action = "retile"
    "#,
    )
    .unwrap();

    // Assert -- the entries take the place of the list of defaults, which
    // the bar merges them over.
    assert_eq!(
        config.mouse.click,
        [ClickBinding::new("clock", "middle", "retile")]
    );
    assert!(config.mouse.enabled);
}

#[test]
fn a_click_entry_whose_value_is_not_text_makes_the_file_invalid() {
    // Act
    let result: Result<BarConfig, _> =
        toml::from_str("[[mouse.click]]\nwidget = \"clock\"\nbutton = 1\n");

    // Assert -- as any other value of the wrong type.
    assert!(result.is_err());
}

#[test]
fn a_key_an_entry_leaves_out_is_missing_and_an_empty_one_is_kept() {
    // Act
    let config: BarConfig =
        toml::from_str("[[mouse.click]]\nwidget = \"clock\"\nbutton = \"\"\n").unwrap();

    // Assert -- and the entry writes back as it was read.
    let entry = &config.mouse.click[0];
    assert_eq!(entry.button.as_deref(), Some(""));
    assert_eq!(entry.action, None);
    let text = toml::to_string(&config.mouse).unwrap();
    let again: BarMouseConfig = toml::from_str(&text).unwrap();
    assert_eq!(again, config.mouse);
}
