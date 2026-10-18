//! Entries with keys left out or written empty: each is left out on its
//! own, said with the keys the file gives it, as it writes them, and with
//! every key it leaves out named at once.

use super::support::with;
use super::*;

/// An entry with only the keys given.
fn entry(widget: Option<&str>, button: Option<&str>, action: Option<&str>) -> ClickBinding {
    ClickBinding {
        widget: widget.map(String::from),
        button: button.map(String::from),
        action: action.map(String::from),
    }
}

/// Why the bar leaves out `binding`, its one entry.
fn why(binding: ClickBinding) -> String {
    let rejected = WidgetClicks::rejected(&with(&[binding]));
    assert_eq!(rejected.len(), 1, "{rejected:?}");
    rejected[0].problem.to_string()
}

#[test]
fn every_key_an_entry_leaves_out_is_named_at_once() {
    // Act
    let (clock, left, retile) = (Some("clock"), Some("left"), Some("retile"));
    let reasons = [
        why(entry(None, left, retile)),
        why(entry(clock, None, retile)),
        why(entry(clock, left, None)),
        why(entry(None, None, retile)),
        why(entry(None, left, None)),
        why(entry(clock, None, None)),
        why(ClickBinding::default()),
    ];

    // Assert
    assert_eq!(
        reasons,
        [
            "it has no widget",
            "it has no button",
            "it has no action",
            "it has no widget and no button",
            "it has no widget and no action",
            "it has no button and no action",
            "it has no widget, no button and no action",
        ]
    );
}

#[test]
fn an_entry_is_said_with_the_keys_it_has() {
    // Act
    let some = WidgetClicks::rejected(&with(&[entry(Some("clock"), None, None)]));
    let none = WidgetClicks::rejected(&with(&[ClickBinding::default()]));

    // Assert
    assert_eq!(
        some[0].to_string(),
        "[[mouse.click]] with widget = \"clock\" is ignored: it has no button and no action"
    );
    assert_eq!(
        none[0].to_string(),
        "[[mouse.click]] with no keys is ignored: it has no widget, no button and no action"
    );
}

#[test]
fn an_empty_value_is_written_as_it_is_and_said_to_be_empty() {
    // Act
    let (clock, left, retile) = (Some("clock"), Some("left"), Some("retile"));
    let lines = [
        WidgetClicks::rejected(&with(&[entry(Some(""), left, retile)])),
        WidgetClicks::rejected(&with(&[entry(clock, Some(""), retile)])),
        WidgetClicks::rejected(&with(&[entry(clock, left, Some(""))])),
    ]
    .map(|rejected| rejected[0].to_string());

    // Assert
    assert_eq!(
        lines,
        [
            "[[mouse.click]] with widget = \"\", button = \"left\" and action = \"retile\" \
             is ignored: the widget is empty",
            "[[mouse.click]] with widget = \"clock\", button = \"\" and action = \"retile\" \
             is ignored: the button is empty",
            "[[mouse.click]] with widget = \"clock\", button = \"left\" and action = \"\" \
             is ignored: the action is empty",
        ]
    );
}

#[test]
fn a_key_left_out_is_named_before_a_value_that_is_empty_or_unknown() {
    // Act
    let reason = why(entry(Some(""), None, Some("fly")));

    // Assert -- once the button is added, the widget comes next.
    assert_eq!(reason, "it has no button");
}
