//! Retile and the toggle tiling floating windows again, but never those
//! the rules float.

use crate::config::{RuleAction, WindowRule};
use crate::id::wid;
use crate::tiler::Input;
use crate::tiler::tests::Harness;
use crate::{Action, WindowEvent};

/// Windows 1 and 2 on one monitor, window 2 floated with focus.
fn floated() -> Harness {
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleFloat);
    h
}

/// Windows 1, 2 and 3 on one monitor, window 3 floated by a rule.
fn rule_floated() -> Harness {
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    if let Some(facts) = h.window(3).facts.as_mut() {
        facts.class = "Tool".into();
    }
    h.step(Input::Rules(vec![WindowRule {
        match_class: Some("Tool".into()),
        match_title: None,
        match_path: None,
        action: RuleAction::Float,
    }]));
    h
}

#[test]
fn a_retile_tiles_the_floating_windows_but_those_the_rules_float() {
    // Arrange -- window 3 floats by a rule, and window 2 by the user.
    let mut h = rule_floated();
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleFloat);

    // Act
    h.action(Action::Retile);

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!(h.workspace(0, 0).floating(), &[wid(3)]);
}

#[test]
fn the_toggle_does_not_tile_a_window_the_rules_float() {
    // Arrange
    let mut h = rule_floated();
    h.event(WindowEvent::Focused { window: wid(3) });

    // Act
    h.action(Action::ToggleFloat);

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!(h.workspace(0, 0).floating(), &[wid(3)]);
}

#[test]
fn a_retile_shows_a_floating_window_monocle_hid_as_a_tile() {
    // Arrange -- tile 1 in monocle hides floating window 2.
    let mut h = floated();
    h.event(WindowEvent::Focused { window: wid(1) });
    h.action(Action::ToggleMonocle);

    // Act
    h.action(Action::Retile);

    // Assert
    assert!(h.workspace(0, 0).floating().is_empty());
    assert!(!h.window(2).cloaked);
}
