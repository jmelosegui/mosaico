//! `action = "none"`: an entry that removes the binding of its widget and
//! button, a default included, which is not an entry the bar leaves out.
//! Deleting an entry, on the other hand, keeps the default.

use super::support::{left, mouse, with};
use super::*;

#[test]
fn none_removes_each_default_and_leaves_the_others() {
    // Arrange
    let defaults = [
        (WidgetKind::Layout, Button::Left, "layout", "left"),
        (WidgetKind::Layout, Button::Right, "layout", "right"),
        (WidgetKind::Paused, Button::Left, "paused", "left"),
    ];

    // Act / Assert -- with no warning or doctor line for the entry.
    for (kind, button, widget, name) in defaults {
        let mouse = with(&[ClickBinding::new(widget, name, ClickBinding::NONE)]);
        let clicks = WidgetClicks::new(&mouse);
        assert_eq!(left(&clicks, kind, button), None, "{widget} {name}");
        assert_eq!(clicks.len(), 2, "{widget} {name}");
        assert_eq!(WidgetClicks::rejected(&mouse), [], "{widget} {name}");
    }
}

#[test]
fn a_widget_whose_every_binding_is_removed_takes_no_clicks() {
    // Arrange
    let mouse = with(&[
        ClickBinding::new("layout", "left", "none"),
        ClickBinding::new("layout", "right", "none"),
        ClickBinding::new("paused", "left", "none"),
    ]);

    // Act
    let clicks = WidgetClicks::new(&mouse);

    // Assert
    assert!(!clicks.clickable(WidgetKind::Layout, BarSide::Left));
    assert!(!clicks.clickable(WidgetKind::Paused, BarSide::Right));
    assert!(clicks.is_empty());
}

#[test]
fn a_later_entry_wins_over_an_earlier_none_and_the_other_way_round() {
    // Arrange
    let bound_again = with(&[
        ClickBinding::new("layout", "left", "none"),
        ClickBinding::new("layout", "left", "goto-workspace-next"),
    ]);
    let removed_again = with(&[
        ClickBinding::new("clock", "middle", "retile"),
        ClickBinding::new("clock", "middle", "none"),
    ]);

    // Act
    let bound = WidgetClicks::new(&bound_again);
    let removed = WidgetClicks::new(&removed_again);

    // Assert
    let layout = left(&bound, WidgetKind::Layout, Button::Left);
    assert_eq!(layout, Some(Action::GoToWorkspaceNext));
    assert_eq!(left(&removed, WidgetKind::Clock, Button::Middle), None);
    assert!(!removed.clickable(WidgetKind::Clock, BarSide::Right));
}

#[test]
fn none_on_a_widget_that_does_not_exist_is_still_left_out() {
    // Act
    let rejected = WidgetClicks::rejected(&with(&[ClickBinding::new("radio", "left", "none")]));

    // Assert
    let problems: Vec<&ClickProblem> = rejected.iter().map(|r| &r.problem).collect();
    assert_eq!(problems, [&ClickProblem::UnknownWidget]);
}

#[test]
fn an_empty_list_or_a_deleted_entry_keeps_the_defaults() {
    // Arrange -- the entries the template writes, with the right click on
    // the layout widget deleted.
    let deleted = with(&[
        ClickBinding::new("layout", "left", "cycle-layout"),
        ClickBinding::new("paused", "left", "toggle-pause"),
    ]);

    // Act / Assert
    for mouse in [mouse("[mouse]\nclick = []\n"), deleted] {
        let clicks = WidgetClicks::new(&mouse);
        assert_eq!(clicks.len(), 3);
        let layout = left(&clicks, WidgetKind::Layout, Button::Right);
        assert_eq!(layout, Some(Action::ToggleMonocle));
    }
}
