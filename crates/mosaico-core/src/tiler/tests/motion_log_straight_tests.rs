//! What the log is told of a motion the tiler ends: as it hides the window,
//! lets go of it, places it at once or reveals it.

use crate::config::WindowRule;
use crate::id::wid;
use crate::tiler::applier::Applier;
use crate::tiler::tests::fake_platform::FakePlatform;
use crate::tiler::{Effect, Platform, Timing, Transition};
use crate::{Action, Rect};

use super::super::{REFRESH, animated};
use super::end_tests::{assert_ended, moving};
use super::starting;

#[test]
fn moving_windows_a_workspace_switch_hides_are_logged_as_hidden() {
    // Arrange
    let (_platform, mut manager) = moving();

    // Act
    manager.handle_action(&Action::GoToWorkspace(2));

    // Assert
    let lines = manager.transition_lines();
    for id in ["0x1", "0x2"] {
        let hidden = format!("transition: {id} stopped where it was, hidden after ");
        assert_eq!(starting(&lines, &hidden), 1, "{lines:#?}");
    }
}

#[test]
fn a_moving_window_the_rules_let_go_of_is_logged_as_let_go() {
    // Arrange
    let (platform, mut manager) = moving();
    platform.edit(1, |w| {
        w.facts.iter_mut().for_each(|f| f.title = "let go".into())
    });
    let rule = WindowRule {
        match_class: None,
        match_title: Some("let go".into()),
        match_path: None,
        manage: false,
    };

    // Act
    manager.reload_rules(vec![rule]);

    // Assert
    let how = "went straight to its target, let go of by the tiler";
    assert_ended(&mut manager, 1, how);
}

#[test]
fn a_moving_window_placed_at_once_is_logged_as_ended_by_that_placement() {
    // Arrange -- the system turns animations off.
    let (platform, mut manager) = moving();
    platform.allow_animations(false);

    // Act
    manager.handle_action(&Action::Retile);

    // Assert
    assert_ended(&mut manager, 1, "ended with a placement that snaps");
}

#[test]
fn a_moving_window_revealed_is_logged_as_sent_straight_to_its_target() {
    // Arrange -- the applier on its own: the manager ends every motion
    // before the tiler reveals any window.
    let mut platform = FakePlatform::new(1, &[(1, 0)]);
    let mut applier = Applier::default();
    applier.set_animations(&animated().animations);
    let window = wid(1);
    let rect = Rect {
        x: 400,
        ..platform.window(1).rect
    };
    let (timing, transition) = (Timing::Now, Transition::Animate);
    let place = Effect::Place {
        window,
        rect,
        timing,
        transition,
    };
    let now = platform.now();
    applier.apply(vec![place], &mut platform, now);

    // Act
    applier.apply(
        vec![Effect::Reveal { window }],
        &mut platform,
        now + REFRESH,
    );

    // Assert
    let lines = applier.take_transition_lines();
    let revealed = "transition: 0x1 went straight to its target, revealed after 16 ms of 180: ";
    assert_eq!(starting(&lines, revealed), 1, "{lines:#?}");
}
