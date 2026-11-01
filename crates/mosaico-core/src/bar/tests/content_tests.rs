use super::*;
use crate::bar::fakes::FakeSystem;
use crate::id::wid;

/// A bar config written the way `bar.toml` is.
fn bar(toml: &str) -> BarConfig {
    toml::from_str(toml).expect("a valid bar.toml")
}

/// What a bar with `config` shows on a monitor in `state`.
fn content(config: &BarConfig, state: &BarState) -> BarContent {
    let readings = SystemReadings::read(&FakeSystem::default(), config, None);
    BarContent::build(config, state, &readings)
}

/// The label of each pill in `items`, and the text of each separator.
fn texts(items: &[BarItem]) -> Vec<String> {
    items
        .iter()
        .map(|item| match item {
            BarItem::Pill { label, .. } => label.clone(),
            BarItem::Separator(text) => text.clone(),
            other => format!("{other:?}"),
        })
        .collect()
}

#[test]
fn the_workspaces_widget_numbers_every_workspace_from_one() {
    // Arrange
    let config = bar("right = []\nleft = [{ type = \"workspaces\" }]");
    let state = BarState {
        workspace_count: 3,
        active_workspace: 1,
        ..BarState::default()
    };

    // Act
    let content = content(&config, &state);

    // Assert
    let labels = vec!["1".to_string(), "2".to_string(), "3".to_string()];
    assert_eq!(
        content.left,
        vec![BarItem::Workspaces { labels, active: 1 }]
    );
}

#[test]
fn the_active_window_widget_shows_the_focused_window() {
    // Arrange
    let config = bar("right = []\nleft = [{ type = \"active_window\" }]");
    let state = BarState {
        focused_window: Some(wid(7)),
        ..BarState::default()
    };

    // Act
    let content = content(&config, &state);

    // Assert
    assert_eq!(content.left, vec![BarItem::WindowIcon(Some(wid(7)))]);
}

#[test]
fn the_active_window_keeps_its_place_and_separators_without_a_window() {
    // Arrange -- unfocused monitors have always drawn both separators.
    let config = bar(r#"
        separator = "|"
        right = []
        left = [{ type = "layout" }, { type = "active_window" }, { type = "layout" }]
    "#);

    // Act
    let content = content(&config, &BarState::default());

    // Assert
    let separator = BarItem::Separator(" | ".into());
    assert_eq!(content.left[1], separator);
    assert_eq!(content.left[2], BarItem::WindowIcon(None));
    assert_eq!(content.left[3], separator);
    assert_eq!(content.left.len(), 5);
}

#[test]
fn workspaces_and_the_active_window_on_the_right_are_plain_pills() {
    // Arrange
    let config = bar(r#"
        left = []
        right = [{ type = "workspaces", icon = "W" }, { type = "active_window" }]
    "#);
    let state = BarState {
        focused_window: Some(wid(7)),
        ..BarState::default()
    };

    // Act
    let content = content(&config, &state);

    // Assert
    assert_eq!(texts(&content.right), ["W", ""]);
}

#[test]
fn each_side_keeps_its_widgets_in_config_order() {
    // Arrange
    let config = bar(r#"
        left = [{ type = "layout" }]
        right = [{ type = "clock", format = "a" }, { type = "clock", format = "b" }]
    "#);

    // Act
    let content = content(&config, &BarState::default());

    // Assert
    assert_eq!(texts(&content.left), ["BSP"]);
    assert_eq!(texts(&content.right), ["a", "b"]);
}

#[test]
fn a_separator_only_goes_between_widgets_that_show() {
    // Arrange -- the update widget in the middle has nothing to show.
    let config = bar(r#"
        separator = "|"
        left = []
        right = [
            { type = "clock", format = "a" },
            { type = "update", icon = "" },
            { type = "clock", format = "b" },
        ]
    "#);

    // Act
    let content = content(&config, &BarState::default());

    // Assert
    assert_eq!(texts(&content.right), ["a", " | ", "b"]);
}

#[test]
fn no_separator_is_added_when_none_is_configured() {
    // Arrange
    let config = bar(r#"
        left = []
        right = [{ type = "clock", format = "a" }, { type = "clock", format = "b" }]
    "#);

    // Act
    let content = content(&config, &BarState::default());

    // Assert
    assert_eq!(texts(&content.right), ["a", "b"]);
}

#[test]
fn a_widget_color_tints_its_text_and_outline() {
    // Arrange
    let config = bar(r##"
        colors = { foreground = "#ffffff", pill_border = "#000000" }
        left = []
        right = [{ type = "clock", format = "a", color = "#ff0000" }]
    "##);

    // Act
    let content = content(&config, &BarState::default());

    // Assert
    let pill = BarItem::Pill {
        icon: None,
        label: "a".into(),
        text_color: "#ff0000".into(),
        border_color: "#ff0000".into(),
    };
    assert_eq!(content.right, vec![pill]);
}

#[test]
fn a_widget_without_a_color_uses_the_bar_colors() {
    // Arrange
    let config = bar(r##"
        colors = { foreground = "#ffffff", pill_border = "#000000" }
        left = []
        right = [{ type = "clock", format = "a" }]
    "##);

    // Act
    let content = content(&config, &BarState::default());

    // Assert
    let pill = BarItem::Pill {
        icon: None,
        label: "a".into(),
        text_color: "#ffffff".into(),
        border_color: "#000000".into(),
    };
    assert_eq!(content.right, vec![pill]);
}
