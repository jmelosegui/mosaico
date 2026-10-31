//! What a rules reload leaves behind when it lets go of a hidden window
//! (#111).

use crate::config::{RuleAction, WindowRule};

use super::*;

/// Rules that let go of `window` and keep the others.
fn rejecting(h: &mut Harness, window: u64) -> Vec<WindowRule> {
    if let Some(facts) = h.window(window).facts.as_mut() {
        facts.class = "Excluded".into();
    }
    vec![WindowRule {
        match_class: Some("Excluded".into()),
        match_title: None,
        match_path: None,
        action: RuleAction::Ignore,
    }]
}

#[test]
fn a_hidden_window_the_platform_no_longer_describes_is_let_go_and_shown() {
    // #111
    // Arrange -- window 2 is cloaked on workspace 1, and the platform no
    // longer sees an application window in it.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.action(Action::SendToWorkspace(2));
    h.window(2).facts = None;

    // Act
    h.step(Input::Rules(Vec::new()));

    // Assert
    assert_eq!(h.tiler.window_count(), 1);
    assert!(
        !h.desktop.windows[&wid(2)].cloaked,
        "window 2 was left cloaked"
    );
}

#[test]
fn letting_go_of_a_hidden_workspaces_monocle_window_turns_monocle_off() {
    // Arrange -- window 2 fills workspace 1 in monocle, alongside windows
    // 1 and 3, when the user switches to workspace 2.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleMonocle);
    h.action(Action::GoToWorkspace(2));
    let rules = rejecting(&mut h, 2);

    // Act
    h.step(Input::Rules(rules));

    // Assert -- otherwise switching back would show a monocle workspace
    // without its window.
    let ws = h.workspace(0, 0);
    assert_eq!(ws.windows(), [wid(1), wid(3)]);
    assert!(!ws.monocle(), "workspace 1 is still in monocle");
    assert_eq!(ws.monocle_window(), None);
}
