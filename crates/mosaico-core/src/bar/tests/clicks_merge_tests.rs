//! The `[[mouse.click]]` entries over the defaults: each replaces the
//! binding of its widget and button, or adds one.

use super::support::{left, mouse, with};
use super::*;

#[test]
fn an_entry_for_the_same_widget_and_button_replaces_the_default() {
    // Arrange
    let mouse = mouse(
        r#"
        [[mouse.click]]
        widget = "layout"
        button = "left"
        action = "goto-workspace-next"
    "#,
    );

    // Act
    let clicks = WidgetClicks::new(&mouse);

    // Assert -- the other defaults stay.
    let layout = |button| left(&clicks, WidgetKind::Layout, button);
    assert_eq!(layout(Button::Left), Some(Action::GoToWorkspaceNext));
    assert_eq!(layout(Button::Right), Some(Action::ToggleMonocle));
    let paused = left(&clicks, WidgetKind::Paused, Button::Left);
    assert_eq!(paused, Some(Action::TogglePause));
}

#[test]
fn an_entry_for_another_widget_or_button_adds_a_binding() {
    // Arrange
    let mouse = mouse(
        r#"
        [[mouse.click]]
        widget = "clock"
        button = "middle"
        action = "retile"

        [[mouse.click]]
        widget = "layout"
        button = "middle"
        action = "goto-workspace-1"
    "#,
    );

    // Act
    let clicks = WidgetClicks::new(&mouse);

    // Assert
    let clock = left(&clicks, WidgetKind::Clock, Button::Middle);
    assert_eq!(clock, Some(Action::Retile));
    let layout = |button| left(&clicks, WidgetKind::Layout, button);
    assert_eq!(layout(Button::Middle), Some(Action::GoToWorkspace(1)));
    assert_eq!(layout(Button::Left), Some(Action::CycleLayout));
    assert!(!clicks.clickable(WidgetKind::Date, BarSide::Left));
}

#[test]
fn of_two_entries_for_the_same_widget_and_button_the_later_one_wins() {
    // Arrange
    let mouse = with(&[
        ClickBinding::new("date", "left", "goto-workspace-prev"),
        ClickBinding::new("date", "left", "goto-workspace-next"),
    ]);

    // Act
    let clicks = WidgetClicks::new(&mouse);

    // Assert
    let date = left(&clicks, WidgetKind::Date, Button::Left);
    assert_eq!(date, Some(Action::GoToWorkspaceNext));
}
