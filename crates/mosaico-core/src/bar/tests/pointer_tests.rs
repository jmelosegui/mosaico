//! Clicks on the bars, as the manager turns them into actions.

use std::time::Instant;

use crate::Action;
use crate::action::Origin;
use crate::bar::fakes::FakeRenderer;
use crate::bar::fakes::layout::{EDGE, GAP};
use crate::bar::{BarState, Button};
use crate::config::BarConfig;

use super::super::tests::bar;
use super::support::{bar_on, click, drawn, label, press, release};

#[test]
fn a_left_click_on_a_label_goes_to_that_workspace_on_the_bars_monitor() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn(BarConfig::default(), &renderer);
    let second = bar_on(&renderer, 1);

    // Act
    let asked = click(&mut manager, second, label(2), label(2));

    // Assert
    assert_eq!(asked, Some((Action::GoToWorkspace(3), Origin::Bar(1))));
}

#[test]
fn a_press_on_one_label_and_a_release_on_another_ask_for_nothing() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn(BarConfig::default(), &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let asked = click(&mut manager, first, label(1), label(3));

    // Assert
    assert_eq!(asked, None);
}

#[test]
fn a_right_click_on_a_label_asks_for_nothing() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn(BarConfig::default(), &renderer);
    let first = bar_on(&renderer, 0);
    let now = Instant::now();

    // Act
    manager.pointer(first, press(Button::Right, label(1)), now);
    let asked = manager.pointer(first, release(Button::Right, label(1)), now);

    // Assert
    assert_eq!(asked, None);
}

#[test]
fn a_click_on_a_widget_asks_for_nothing() {
    // Arrange -- only a layout pill on the left.
    let renderer = FakeRenderer::default();
    let mut manager = drawn(bar("left = [{ type = \"layout\" }]"), &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let asked = click(&mut manager, first, EDGE + 1, EDGE + 1);

    // Assert
    assert_eq!(asked, None);
}

#[test]
fn a_redraw_between_press_and_release_still_clicks_the_label() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn(BarConfig::default(), &renderer);
    let first = bar_on(&renderer, 0);
    let now = Instant::now();
    manager.pointer(first, press(Button::Left, label(4)), now);

    // Act -- the clock ticks and the bar is drawn again.
    manager.update(&[BarState::default(), BarState::default()]);
    let asked = manager.pointer(first, release(Button::Left, label(4)), now);

    // Assert
    assert_eq!(asked, Some((Action::GoToWorkspace(5), Origin::Bar(0))));
}

#[test]
fn the_label_after_the_window_icon_still_names_its_workspace() {
    // Arrange -- the icon comes first and takes room only with a window.
    let renderer = FakeRenderer::default();
    let config = bar(r#"
        left = [{ type = "active_window" }, { type = "workspaces" }]
    "#);
    let mut manager = drawn(config, &renderer);
    let first = bar_on(&renderer, 0);

    // Act -- with no window focused, the labels start a gap after it.
    let x = label(0) + GAP;
    let asked = click(&mut manager, first, x, x);

    // Assert
    assert_eq!(asked, Some((Action::GoToWorkspace(1), Origin::Bar(0))));
}
