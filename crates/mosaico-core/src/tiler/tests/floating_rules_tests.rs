//! Windows the rules float, and rules reloads moving windows between
//! tiled and floating.

use crate::config::{RuleAction, WindowRule};
use crate::id::wid;
use crate::tiler::Input;
use crate::tiler::tests::{FakeWindow, Harness};
use crate::{Action, Rect, WindowEvent};

/// A rule that does `action` with windows of class `Tool`.
fn tool_rule(action: RuleAction) -> Vec<WindowRule> {
    vec![WindowRule {
        match_class: Some("Tool".into()),
        match_title: None,
        match_path: None,
        action,
    }]
}

/// Makes window `id` a `Tool`.
fn make_tool(h: &mut Harness, id: u64) {
    if let Some(facts) = h.window(id).facts.as_mut() {
        facts.class = "Tool".into();
    }
}

#[test]
fn a_window_the_rules_float_is_centered_at_its_own_size_with_focus() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);
    h.step(Input::Rules(tool_rule(RuleAction::Float)));
    let mut tool = FakeWindow::app(0);
    tool.rect = Rect::new(10, 10, 400, 300);
    if let Some(facts) = tool.facts.as_mut() {
        facts.class = "Tool".into();
    }
    h.desktop.windows.insert(wid(2), tool);

    // Act
    h.event(WindowEvent::Created { window: wid(2) });

    // Assert
    let area = h.tiler.monitors()[0].work_area();
    assert_eq!(h.active(0), vec![1]);
    assert_eq!(h.workspace(0, 0).floating(), &[wid(2)]);
    let rect = h.rect_of(2);
    assert_eq!((rect.width, rect.height), (400, 300));
    assert_eq!(
        (rect.center_x(), rect.center_y()),
        (area.center_x(), area.center_y())
    );
    assert_eq!(h.focus_request(), Some(2));
}

#[test]
fn a_rules_reload_floats_and_tiles_what_it_changes_and_leaves_a_toggled_float() {
    // Arrange -- window 3 was floated by the user.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    make_tool(&mut h, 2);
    h.event(WindowEvent::Focused { window: wid(3) });
    h.action(Action::ToggleFloat);

    // Act
    h.step(Input::Rules(tool_rule(RuleAction::Float)));
    let floated = h.workspace(0, 0).floating().to_vec();
    h.step(Input::Rules(tool_rule(RuleAction::Tile)));

    // Assert
    assert_eq!(floated, [wid(3), wid(2)]);
    assert_eq!(h.workspace(0, 0).floating(), &[wid(3)]);
    assert_eq!(h.active(0), vec![1, 2]);
}
