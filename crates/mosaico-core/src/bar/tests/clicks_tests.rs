//! What clicks on each kind of widget run by default, and with the mouse
//! off.

use super::support::{left, mouse};
use super::*;

#[test]
fn by_default_the_layout_widget_cycles_the_layout_and_toggles_monocle() {
    // Act
    let clicks = WidgetClicks::new(&BarMouseConfig::default());

    // Assert
    let layout = |button| left(&clicks, WidgetKind::Layout, button);
    assert_eq!(layout(Button::Left), Some(Action::CycleLayout));
    assert_eq!(layout(Button::Right), Some(Action::ToggleMonocle));
    assert_eq!(layout(Button::Middle), None);
}

#[test]
fn by_default_a_left_click_on_the_paused_widget_resumes_the_hotkeys() {
    // Act
    let clicks = WidgetClicks::new(&BarMouseConfig::default());

    // Assert -- the paused widget is a pill on either side.
    let paused = |side, button| clicks.action(WidgetKind::Paused, side, button);
    let resume = Some(Action::TogglePause);
    assert_eq!(paused(BarSide::Right, Button::Left), resume);
    assert_eq!(paused(BarSide::Left, Button::Left), resume);
    assert_eq!(paused(BarSide::Right, Button::Right), None);
}

#[test]
fn by_default_no_other_widget_takes_clicks() {
    // Act
    let clicks = WidgetClicks::new(&BarMouseConfig::default());

    // Assert
    let clickable: Vec<WidgetKind> = WidgetKind::ALL
        .into_iter()
        .filter(|&kind| clicks.clickable(kind, BarSide::Left))
        .collect();
    assert_eq!(clickable, [WidgetKind::Layout, WidgetKind::Paused]);
}

#[test]
fn the_defaults_are_all_bindings_a_bar_can_run() {
    // Act
    let rejected = WidgetClicks::rejected(&BarMouseConfig::default());

    // Assert
    assert_eq!(rejected, []);
}

#[test]
fn a_file_without_click_entries_has_the_defaults() {
    // Act
    let mouse = mouse("[mouse]\nscroll = \"none\"\n");

    // Assert
    let defaults = WidgetClicks::new(&BarMouseConfig::default());
    assert_eq!(WidgetClicks::new(&mouse), defaults);
}

#[test]
fn with_the_mouse_off_no_click_runs_anything() {
    // Act
    let clicks = WidgetClicks::new(&mouse("[mouse]\nenabled = false\n"));

    // Assert
    for kind in WidgetKind::ALL {
        assert!(!clicks.clickable(kind, BarSide::Left), "{kind}");
        assert!(!clicks.clickable(kind, BarSide::Right), "{kind}");
    }
}

#[test]
fn the_workspaces_widget_takes_its_clicks_on_the_left_alone() {
    // Arrange -- on the right it is a pill with its icon, and no labels.
    let mouse = mouse(
        r#"
        [[mouse.click]]
        widget = "workspaces"
        button = "right"
        action = "cycle-layout"
    "#,
    );

    // Act
    let clicks = WidgetClicks::new(&mouse);

    // Assert
    let workspaces = |side| clicks.action(WidgetKind::Workspaces, side, Button::Right);
    assert_eq!(workspaces(BarSide::Left), Some(Action::CycleLayout));
    assert_eq!(workspaces(BarSide::Right), None);
    assert!(!clicks.clickable(WidgetKind::Workspaces, BarSide::Right));
}

#[test]
fn the_actions_offered_for_a_click_are_all_ones_the_bar_runs() {
    // Act
    let actions = WidgetClicks::actions();

    // Assert
    assert!(actions.contains(&Action::CycleLayout));
    assert!(actions.contains(&Action::Retile));
    for action in actions {
        let binding = ClickBinding::new("layout", "left", &action.to_string());
        assert!(
            WidgetClicks::rejected(&BarMouseConfig {
                click: vec![binding],
                ..BarMouseConfig::default()
            })
            .is_empty()
        );
    }
}
