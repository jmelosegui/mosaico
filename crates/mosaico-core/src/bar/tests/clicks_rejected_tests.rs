//! The `[[mouse.click]]` entries a bar cannot run: each is left out on
//! its own, as if the file did not have it, and said why.

use super::support::with;
use super::*;

/// Why the bar leaves out each of the entries of `mouse` it leaves out.
fn problems(mouse: &BarMouseConfig) -> Vec<ClickProblem> {
    let rejected = WidgetClicks::rejected(mouse);
    rejected.into_iter().map(|r| r.problem).collect()
}

#[test]
fn an_entry_naming_a_widget_that_does_not_exist_is_left_out() {
    // Arrange
    let mouse = with(&[
        ClickBinding::new("volume", "left", "retile"),
        ClickBinding::new("clock", "left", "retile"),
    ]);

    // Act
    let clicks = WidgetClicks::new(&mouse);

    // Assert -- the entry after it still applies.
    assert_eq!(problems(&mouse), [ClickProblem::UnknownWidget]);
    let clock = clicks.action(WidgetKind::Clock, BarSide::Right, Button::Left);
    assert_eq!(clock, Some(Action::Retile));
}

#[test]
fn an_entry_naming_an_unknown_button_or_action_is_left_out() {
    // Arrange
    let mouse = with(&[
        ClickBinding::new("clock", "double", "retile"),
        ClickBinding::new("clock", "left", "fly"),
        ClickBinding::new("clock", "right", "goto-workspace-9"),
    ]);

    // Act
    let problems = problems(&mouse);

    // Assert
    assert_eq!(
        problems,
        [
            ClickProblem::UnknownButton,
            ClickProblem::UnknownAction("unknown action: fly".into()),
            ClickProblem::UnknownAction("workspace must be 1-8, got 9".into()),
        ]
    );
    assert!(!WidgetClicks::new(&mouse).clickable(WidgetKind::Clock, BarSide::Right));
}

#[test]
fn an_entry_with_a_key_missing_is_left_out_and_the_file_still_loads() {
    // Arrange
    let toml = "[[mouse.click]]\nwidget = \"clock\"\naction = \"retile\"\n";

    // Act
    let config: crate::BarConfig = toml::from_str(toml).unwrap();

    // Assert
    let missing = ClickProblem::Missing(vec!["button"]);
    assert_eq!(problems(&config.mouse), [missing]);
}

#[test]
fn an_action_about_the_focused_window_is_rejected_and_the_default_stays() {
    // Arrange -- each would act on a window of another monitor when the
    // bar of one is clicked.
    let names = [
        "close-focused",
        "minimize-focused",
        "send-to-workspace-2",
        "focus-left",
        "move-right",
    ];

    // Act / Assert
    for name in names {
        let mouse = with(&[ClickBinding::new("layout", "left", name)]);
        assert_eq!(problems(&mouse), [ClickProblem::FocusedWindow], "{name}");
        let layout =
            WidgetClicks::new(&mouse).action(WidgetKind::Layout, BarSide::Left, Button::Left);
        assert_eq!(layout, Some(Action::CycleLayout), "{name}");
    }
}

#[test]
fn an_action_on_a_monitor_or_the_whole_daemon_may_be_bound() {
    // Arrange
    let allowed = [
        ("cycle-layout", Action::CycleLayout),
        ("toggle-monocle", Action::ToggleMonocle),
        ("goto-workspace-3", Action::GoToWorkspace(3)),
        ("goto-workspace-next", Action::GoToWorkspaceNext),
        ("goto-workspace-prev", Action::GoToWorkspacePrev),
        ("toggle-pause", Action::TogglePause),
        ("retile", Action::Retile),
    ];

    // Act / Assert
    for (name, action) in allowed {
        let mouse = with(&[ClickBinding::new("ram", "middle", name)]);
        assert_eq!(problems(&mouse), [], "{name}");
        let ram = WidgetClicks::new(&mouse).action(WidgetKind::Ram, BarSide::Right, Button::Middle);
        assert_eq!(ram, Some(action), "{name}");
    }
}

#[test]
fn a_rejected_entry_reads_as_the_file_writes_it_with_the_reason() {
    // Arrange
    let mouse = with(&[ClickBinding::new("layout", "left", "close-focused")]);

    // Act
    let text = WidgetClicks::rejected(&mouse)[0].to_string();

    // Assert
    assert!(
        text.starts_with(
            "[[mouse.click]] with widget = \"layout\", button = \"left\" and \
             action = \"close-focused\" is ignored: the action is about the focused window"
        ),
        "{text}"
    );
}

#[test]
fn the_entries_left_out_come_in_the_order_the_file_lists_them() {
    // Arrange
    let mouse = with(&[
        ClickBinding::new("clock", "left", "close-focused"),
        ClickBinding::new("date", "left", "retile"),
        ClickBinding::new("radio", "left", "retile"),
    ]);

    // Act
    let rejected = WidgetClicks::rejected(&mouse);

    // Assert
    let widgets: Vec<&str> = rejected
        .iter()
        .filter_map(|r| r.binding.widget.as_deref())
        .collect();
    assert_eq!(widgets, ["clock", "radio"]);
}
