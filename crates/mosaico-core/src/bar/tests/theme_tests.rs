//! The colors the bars are drawn in after the theme changes.

use super::tests::{bar, left, start};
use crate::bar::fakes::BarEvent::{self, Drawn};
use crate::bar::fakes::{FakeRenderer, FakeSystem};
use crate::bar::{BarItem, BarState};
use crate::config::{BarConfig, CatppuccinFlavor, Theme};

const LATTE: Theme = Theme::Catppuccin(CatppuccinFlavor::Latte);

/// Latte's red. Mocha's, which the bars start with, is `#f38ba8`.
const LATTE_RED: &str = "#d20f39";

/// A monitor whose hotkeys are paused, so the paused widget shows.
fn paused() -> BarState {
    BarState {
        paused: true,
        ..BarState::default()
    }
}

/// The text and outline colors of each pill labelled `label` that was
/// drawn.
fn pill_colors(events: Vec<BarEvent>, label: &str) -> Vec<[String; 2]> {
    events
        .into_iter()
        .filter_map(|event| match event {
            Drawn { content, .. } => Some(content),
            _ => None,
        })
        .flat_map(|content| content.left.into_iter().chain(content.right))
        .filter_map(|item| match item {
            BarItem::Pill {
                label: drawn,
                text_color,
                border_color,
            } if drawn == label => Some([text_color, border_color]),
            _ => None,
        })
        .collect()
}

#[test]
fn a_widget_color_given_by_name_follows_a_theme_change() {
    // Arrange -- #109: the default paused widget is "red", and the bars
    // start under Mocha.
    let renderer = FakeRenderer::default();
    let mut manager = start(
        BarConfig::default(),
        &[left()],
        &renderer,
        &FakeSystem::default(),
    );

    // Act
    manager.resolve_colors(LATTE);
    manager.update(&[paused()]);

    // Assert
    assert_eq!(
        pill_colors(renderer.take(), "PAUSED"),
        [[LATTE_RED, LATTE_RED]]
    );
}

#[test]
fn a_widget_color_given_by_name_follows_the_theme_after_a_display_change() {
    // Arrange -- #109
    let renderer = FakeRenderer::default();
    let mut manager = start(
        BarConfig::default(),
        &[left()],
        &renderer,
        &FakeSystem::default(),
    );

    // Act
    manager.rebuild_for_monitors(vec![left()], LATTE);
    manager.update(&[paused()]);

    // Assert
    assert_eq!(
        pill_colors(renderer.take(), "PAUSED"),
        [[LATTE_RED, LATTE_RED]]
    );
}

#[test]
fn a_widget_color_given_as_hex_stays_after_a_theme_change() {
    // Arrange
    let renderer = FakeRenderer::default();
    let config = bar(r##"
        left = []
        right = [{ type = "paused", color = "#123456" }]
    "##);
    let mut manager = start(config, &[left()], &renderer, &FakeSystem::default());

    // Act
    manager.resolve_colors(LATTE);
    manager.update(&[paused()]);

    // Assert
    assert_eq!(
        pill_colors(renderer.take(), "PAUSED"),
        [["#123456", "#123456"]]
    );
}

#[test]
fn a_bar_color_given_by_name_still_follows_a_theme_change() {
    // Arrange -- the [colors] table followed the theme before #109.
    let renderer = FakeRenderer::default();
    let config = bar(r#"
        colors = { foreground = "red" }
        left = []
        right = [{ type = "layout" }]
    "#);
    let mut manager = start(config, &[left()], &renderer, &FakeSystem::default());

    // Act
    manager.resolve_colors(LATTE);
    manager.update(&[]);

    // Assert
    let colors = pill_colors(renderer.take(), "BSP");
    assert_eq!(colors.len(), 1);
    assert_eq!(colors[0][0], LATTE_RED);
}
