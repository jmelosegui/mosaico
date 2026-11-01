//! What the tiler remembers about the windows it hid, once they leave
//! the layout (#92).
//!
//! A window that leaves the layout while hidden has to be forgotten,
//! however it leaves: otherwise a new window that gets its id is hidden
//! the way the old one was.

use crate::config::{Config, HidingBehaviour, RuleAction, WindowRule};

use super::*;

/// A harness that hides windows with "hide".
fn hiding_with_hide(windows: &[(u64, usize)]) -> Harness {
    let mut settings = Settings::with_gap(8);
    settings.layout.hiding = HidingBehaviour::Hide;
    Harness::with_settings(settings, 1, windows)
}

/// Reloads a configuration that minimizes windows to hide them.
fn reload_with_minimize(h: &mut Harness) {
    let mut config = Config::with_gap(8);
    config.layout.hiding = HidingBehaviour::Minimize;
    h.step(Input::Config(Box::new(config)));
}

#[test]
fn a_new_window_given_a_closed_windows_id_is_hidden_the_configured_way() {
    // Arrange -- window 1 closes while hidden with "hide"; after the
    // config changes to "minimize", a new window gets its id.
    let mut h = hiding_with_hide(&[(1, 0)]);
    h.action(Action::GoToWorkspace(2));
    h.event(WindowEvent::Destroyed { window: wid(1) });
    reload_with_minimize(&mut h);
    h.open(1, 0);

    // Act
    let effects = h.action(Action::GoToWorkspace(1)).to_vec();

    // Assert
    assert!(
        effects.contains(&Effect::Hide {
            window: wid(1),
            how: HidingBehaviour::Minimize,
        }),
        "{effects:?}"
    );
}

#[test]
fn a_new_window_given_the_id_of_one_let_go_while_hidden_is_hidden_the_configured_way() {
    // Arrange -- a rules reload lets go of windows 1 and 2 while they are
    // hidden with "hide". Window 1 then closes, out of the layout, and
    // after the config changes to "minimize" a new window gets its id.
    let mut h = hiding_with_hide(&[(1, 0), (2, 0)]);
    h.action(Action::GoToWorkspace(2));
    h.step(Input::Rules(Vec::new()));
    h.event(WindowEvent::Destroyed { window: wid(1) });
    reload_with_minimize(&mut h);
    h.open(1, 0);

    // Act
    let effects = h.action(Action::GoToWorkspace(1)).to_vec();

    // Assert
    assert!(
        effects.contains(&Effect::Hide {
            window: wid(1),
            how: HidingBehaviour::Minimize,
        }),
        "{effects:?}"
    );
}

#[test]
fn a_window_let_go_by_the_rules_while_hidden_is_hidden_the_configured_way_once_taken_in_again() {
    // Arrange -- new rules let go of window 1 while it is hidden with
    // "hide". Then the rules take it back, the config changes to
    // "minimize", and the application shows window 1 again.
    let mut h = hiding_with_hide(&[(1, 0), (2, 0)]);
    h.action(Action::GoToWorkspace(2));
    if let Some(facts) = h.window(1).facts.as_mut() {
        facts.class = "Excluded".into();
    }
    h.step(Input::Rules(vec![WindowRule {
        match_class: Some("Excluded".into()),
        match_title: None,
        match_path: None,
        action: RuleAction::Ignore,
    }]));
    h.step(Input::Rules(Vec::new()));
    reload_with_minimize(&mut h);
    h.window(1).visible = true;
    h.event(WindowEvent::Created { window: wid(1) });

    // Act
    let effects = h.action(Action::GoToWorkspace(1)).to_vec();

    // Assert
    assert!(
        effects.contains(&Effect::Hide {
            window: wid(1),
            how: HidingBehaviour::Minimize,
        }),
        "{effects:?}"
    );
}
