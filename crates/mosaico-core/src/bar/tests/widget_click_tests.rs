//! Clicks on the bars' widgets, as the manager turns them into the
//! actions their buttons are bound to.

use crate::Action;
use crate::action::Origin;
use crate::bar::fakes::FakeRenderer;
use crate::bar::fakes::layout::{EDGE, GAP, PILL};
use crate::bar::{BarState, Button};

use super::super::tests::bar;
use super::support::{FIRST_ON_THE_RIGHT, bar_on, clicked, drawn, drawn_with};

#[test]
fn a_left_click_on_the_layout_widget_cycles_the_layout_of_the_bars_monitor() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn(bar("left = [{ type = \"layout\" }]"), &renderer);
    let second = bar_on(&renderer, 1);

    // Act
    let asked = clicked(&mut manager, second, Button::Left, EDGE + 1);

    // Assert
    assert_eq!(asked, Some((Action::CycleLayout, Origin::Bar(1))));
}

#[test]
fn a_right_click_on_the_layout_widget_toggles_monocle_and_a_middle_one_nothing() {
    // Arrange
    let renderer = FakeRenderer::default();
    let mut manager = drawn(bar("left = [{ type = \"layout\" }]"), &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let right = clicked(&mut manager, first, Button::Right, EDGE + 1);
    let middle = clicked(&mut manager, first, Button::Middle, EDGE + 1);

    // Assert
    assert_eq!(right, Some((Action::ToggleMonocle, Origin::Bar(0))));
    assert_eq!(middle, None);
}

#[test]
fn a_left_click_on_the_paused_widget_resumes_the_hotkeys() {
    // Arrange -- the widget shows while the hotkeys are paused.
    let renderer = FakeRenderer::default();
    let paused = BarState {
        paused: true,
        ..BarState::default()
    };
    let config = bar("left = []\nright = [{ type = \"paused\" }]");
    let mut manager = drawn_with(config, &[paused.clone(), paused], &renderer);
    let second = bar_on(&renderer, 1);

    // Act
    let asked = clicked(&mut manager, second, Button::Left, FIRST_ON_THE_RIGHT);

    // Assert
    assert_eq!(asked, Some((Action::TogglePause, Origin::Bar(1))));
}

#[test]
fn while_the_hotkeys_work_the_paused_widget_is_not_there_to_click() {
    // Arrange
    let renderer = FakeRenderer::default();
    let config = bar("left = []\nright = [{ type = \"paused\" }]");
    let mut manager = drawn(config, &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let asked = clicked(&mut manager, first, Button::Left, FIRST_ON_THE_RIGHT);

    // Assert
    assert_eq!(asked, None);
}

#[test]
fn a_binding_runs_on_every_widget_of_its_kind() {
    // Arrange -- two clocks on the right, and a middle click bound on them.
    let renderer = FakeRenderer::default();
    let config = bar(r#"
        left = []
        right = [{ type = "clock" }, { type = "clock", format = "%H" }]

        [[mouse.click]]
        widget = "clock"
        button = "middle"
        action = "retile"
    "#);
    let mut manager = drawn(config, &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let second_clock = FIRST_ON_THE_RIGHT - PILL - GAP;
    let asked = clicked(&mut manager, first, Button::Middle, second_clock);

    // Assert
    assert_eq!(asked, Some((Action::Retile, Origin::Bar(0))));
}

#[test]
fn an_entry_runs_in_place_of_the_default_and_a_rejected_one_leaves_it() {
    // Arrange
    let renderer = FakeRenderer::default();
    let config = bar(r#"
        left = [{ type = "layout" }]

        [[mouse.click]]
        widget = "layout"
        button = "left"
        action = "goto-workspace-next"

        [[mouse.click]]
        widget = "layout"
        button = "right"
        action = "close-focused"
    "#);
    let mut manager = drawn(config, &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let left = clicked(&mut manager, first, Button::Left, EDGE + 1);
    let right = clicked(&mut manager, first, Button::Right, EDGE + 1);

    // Assert
    assert_eq!(left, Some((Action::GoToWorkspaceNext, Origin::Bar(0))));
    assert_eq!(right, Some((Action::ToggleMonocle, Origin::Bar(0))));
}

#[path = "widget_click_where_tests.rs"]
mod where_tests;

#[path = "widget_click_unbind_tests.rs"]
mod unbind_tests;
