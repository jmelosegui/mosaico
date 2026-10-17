//! Where on a bar a widget's click is taken: the workspaces widget, its
//! labels and the space around them, a press and a release on two
//! widgets, and bars whose clicks run nothing.

use super::super::support::{click, label};
use super::*;

/// The default bar, with `action` bound to `button` on the workspaces
/// widget.
fn workspaces_bound(button: &str, action: &str) -> crate::config::BarConfig {
    let entry = format!(
        "[[mouse.click]]\nwidget = \"workspaces\"\nbutton = \"{button}\"\naction = \"{action}\"\n"
    );
    bar(&entry)
}

#[test]
fn a_right_click_on_a_label_runs_what_the_workspaces_widget_binds_it_to() {
    // Arrange -- a label is part of the workspaces widget.
    let renderer = FakeRenderer::default();
    let mut manager = drawn(workspaces_bound("right", "cycle-layout"), &renderer);
    let second = bar_on(&renderer, 1);

    // Act
    let right = clicked(&mut manager, second, Button::Right, label(2));
    let left = clicked(&mut manager, second, Button::Left, label(2));

    // Assert -- the left button still goes to the label's workspace.
    assert_eq!(right, Some((Action::CycleLayout, Origin::Bar(1))));
    assert_eq!(left, Some((Action::GoToWorkspace(3), Origin::Bar(1))));
}

#[test]
fn a_click_beside_the_labels_runs_the_workspaces_widgets_binding() {
    // Arrange -- the container starts a little before its first label.
    let renderer = FakeRenderer::default();
    let mut manager = drawn(workspaces_bound("left", "toggle-monocle"), &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let asked = clicked(&mut manager, first, Button::Left, EDGE + 1);

    // Assert
    assert_eq!(asked, Some((Action::ToggleMonocle, Origin::Bar(0))));
}

#[test]
fn the_workspaces_widget_on_the_right_takes_no_click_even_with_a_binding() {
    // Arrange -- there it is a pill with its icon, and no labels.
    let renderer = FakeRenderer::default();
    let config = bar(r#"
        left = []
        right = [{ type = "workspaces" }]

        [[mouse.click]]
        widget = "workspaces"
        button = "left"
        action = "cycle-layout"
    "#);
    let mut manager = drawn(config, &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let asked = clicked(&mut manager, first, Button::Left, FIRST_ON_THE_RIGHT);

    // Assert
    assert_eq!(asked, None);
}

#[test]
fn a_press_on_one_widget_and_a_release_on_another_ask_for_nothing() {
    // Arrange -- two layout pills, side by side.
    let renderer = FakeRenderer::default();
    let config = bar("left = [{ type = \"layout\" }, { type = \"layout\" }]");
    let mut manager = drawn(config, &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let asked = click(&mut manager, first, EDGE + 1, EDGE + PILL + GAP + 1);

    // Assert
    assert_eq!(asked, None);
}

#[test]
fn a_click_on_the_bar_of_a_monitor_the_tiler_has_no_state_for_runs_nothing() {
    // Arrange -- the tiler knows one monitor, and the bars are on two;
    // pausing would act on the whole daemon, whichever bar asked.
    let renderer = FakeRenderer::default();
    let config = bar(r#"
        left = [{ type = "clock" }]

        [[mouse.click]]
        widget = "clock"
        button = "left"
        action = "toggle-pause"
    "#);
    let mut manager = drawn_with(config, &[BarState::default()], &renderer);
    let (first, second) = (bar_on(&renderer, 0), bar_on(&renderer, 1));

    // Act
    let unknown = clicked(&mut manager, second, Button::Left, EDGE + 1);
    let known = clicked(&mut manager, first, Button::Left, EDGE + 1);

    // Assert
    assert_eq!(unknown, None);
    assert_eq!(known, Some((Action::TogglePause, Origin::Bar(0))));
}

#[test]
fn with_the_mouse_off_a_click_on_a_widget_asks_for_nothing() {
    // Arrange
    let renderer = FakeRenderer::default();
    let config = bar("left = [{ type = \"layout\" }]\n\n[mouse]\nenabled = false\n");
    let mut manager = drawn(config, &renderer);
    let first = bar_on(&renderer, 0);

    // Act
    let asked = clicked(&mut manager, first, Button::Left, EDGE + 1);

    // Assert
    assert_eq!(asked, None);
}
