//! Clicks whose binding `action = "none"` removes ask for nothing, and
//! the left click on a workspace label, which is no binding, stays.

use super::super::support::label;
use super::*;

#[test]
fn a_click_whose_binding_none_removes_asks_for_nothing() {
    // Arrange -- the right click keeps its default.
    let renderer = FakeRenderer::default();
    let config = bar(r#"
        left = [{ type = "layout" }]

        [[mouse.click]]
        widget = "layout"
        button = "left"
        action = "none"
    "#);
    let mut manager = drawn(config, &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let left = clicked(&mut manager, first, Button::Left, EDGE + 1);
    let right = clicked(&mut manager, first, Button::Right, EDGE + 1);

    // Assert
    assert_eq!(left, None);
    assert_eq!(right, Some((Action::ToggleMonocle, Origin::Bar(0))));
}

#[test]
fn none_on_the_workspaces_widget_leaves_the_left_click_on_a_label() {
    // Arrange
    let renderer = FakeRenderer::default();
    let config = bar(r#"
        [[mouse.click]]
        widget = "workspaces"
        button = "left"
        action = "none"
    "#);
    let mut manager = drawn(config, &renderer);
    let second = bar_on(&renderer, 1);

    // Act
    let asked = clicked(&mut manager, second, Button::Left, label(2));

    // Assert
    assert_eq!(asked, Some((Action::GoToWorkspace(3), Origin::Bar(1))));
}
