//! Reloading the window rules while windows are hidden (#111).
//!
//! A window hidden by a workspace switch is out of view because the
//! tiler put it there, so only the rules may let go of it, and a window
//! they let go of must not stay hidden.

use crate::config::{HidingBehaviour, WindowRule, WorkspaceMode};

use super::*;

/// Windows 1 and 2 on workspace 1, then window 1 sent to workspace 2,
/// which hides window 2 `how` on workspace 1.
fn window_2_hidden(how: HidingBehaviour) -> Harness {
    let mut settings = Settings::default();
    settings.layout.hiding = how;
    settings.workspaces.mode = WorkspaceMode::PerMonitor;
    let mut h = Harness::with_settings(settings, 1, &[(1, 0), (2, 0)]);
    h.action(Action::SendToWorkspace(2));
    h
}

/// Rules that let go of window 2 and keep window 1.
fn rejecting_window_2(h: &mut Harness) -> Vec<WindowRule> {
    if let Some(facts) = h.window(2).facts.as_mut() {
        facts.class = "Excluded".into();
    }
    vec![WindowRule {
        match_class: Some("Excluded".into()),
        match_title: None,
        match_path: None,
        manage: false,
    }]
}

/// Asserts that nothing at all was let go, shown or redrawn.
fn assert_nothing_changed(h: &Harness) {
    assert!(
        h.effects.is_empty(),
        "the unchanged reload changed something: {:?}",
        h.effects
    );
}

/// Whether the last step showed or revealed any window.
fn showed_anything(h: &Harness) -> bool {
    h.effects
        .iter()
        .any(|e| matches!(e, Effect::Show { .. } | Effect::Reveal { .. }))
}

#[test]
fn a_cloaked_window_is_still_managed_after_the_rules_reload() {
    // #111
    // Arrange -- the rules are loaded again unchanged, as when
    // user-rules.toml is saved, while window 2 is cloaked.
    let mut h = window_2_hidden(HidingBehaviour::Cloak);
    h.step(Input::Rules(Vec::new()));
    assert_nothing_changed(&h);

    // Act
    h.action(Action::GoToWorkspace(1));

    // Assert
    assert_eq!(h.active(0), vec![2]);
    assert!(
        !h.desktop.windows[&wid(2)].cloaked,
        "window 2 is still cloaked"
    );
}

#[test]
fn a_hidden_window_is_still_managed_after_the_rules_reload() {
    // #111
    // Arrange
    let mut h = window_2_hidden(HidingBehaviour::Hide);
    h.step(Input::Rules(Vec::new()));
    assert_nothing_changed(&h);

    // Act
    h.action(Action::GoToWorkspace(1));

    // Assert
    assert_eq!(h.active(0), vec![2]);
    assert!(
        h.desktop.windows[&wid(2)].visible,
        "window 2 is still hidden"
    );
}

#[test]
fn a_cloaked_window_the_new_rules_reject_is_let_go_and_shown() {
    // #111
    // Arrange
    let mut h = window_2_hidden(HidingBehaviour::Cloak);
    let rules = rejecting_window_2(&mut h);

    // Act
    h.step(Input::Rules(rules));

    // Assert -- nothing manages it any more, so it must not stay cloaked.
    assert_eq!(h.tiler.window_count(), 1);
    assert!(
        !h.desktop.windows[&wid(2)].cloaked,
        "window 2 was left cloaked"
    );
    assert!(h.effects.contains(&Effect::Show {
        window: wid(2),
        how: HidingBehaviour::Cloak,
    }));
}

#[test]
fn a_hidden_window_the_new_rules_reject_is_let_go_and_shown() {
    // #111
    // Arrange
    let mut h = window_2_hidden(HidingBehaviour::Hide);
    let rules = rejecting_window_2(&mut h);

    // Act
    h.step(Input::Rules(rules));

    // Assert
    assert_eq!(h.tiler.window_count(), 1);
    assert!(
        h.desktop.windows[&wid(2)].visible,
        "window 2 was left hidden"
    );
    assert!(h.effects.contains(&Effect::Show {
        window: wid(2),
        how: HidingBehaviour::Hide,
    }));
}

#[test]
fn a_minimized_window_the_new_rules_reject_is_let_go_and_restored() {
    // #111
    // Arrange
    let mut h = window_2_hidden(HidingBehaviour::Minimize);
    let rules = rejecting_window_2(&mut h);

    // Act
    h.step(Input::Rules(rules));

    // Assert -- like a window in view, it is left as the user would see
    // it had the tiler never hidden it.
    assert_eq!(h.tiler.window_count(), 1);
    assert!(
        !h.desktop.windows[&wid(2)].minimized,
        "window 2 was left minimized"
    );
    assert!(h.effects.contains(&Effect::Show {
        window: wid(2),
        how: HidingBehaviour::Minimize,
    }));
}

#[test]
fn a_minimized_window_is_still_managed_after_the_rules_reload() {
    // #111
    // Arrange
    let mut h = window_2_hidden(HidingBehaviour::Minimize);
    h.step(Input::Rules(Vec::new()));
    assert_nothing_changed(&h);

    // Act
    h.action(Action::GoToWorkspace(1));

    // Assert
    assert_eq!(h.active(0), vec![2]);
    assert!(!h.desktop.windows[&wid(2)].minimized);
}

#[test]
fn a_window_in_view_the_new_rules_reject_is_let_go_as_before() {
    // Arrange -- window 2 is on screen.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let rules = rejecting_window_2(&mut h);

    // Act
    h.step(Input::Rules(rules));

    // Assert -- it is in view already, so nothing shows it.
    assert_eq!(h.active(0), vec![1]);
    assert!(h.effects.contains(&Effect::Undecorate { window: wid(2) }));
    assert!(!showed_anything(&h), "{:?}", h.effects);
}

#[test]
fn a_window_gone_from_view_without_a_report_is_let_go_as_before() {
    // Arrange -- window 2 is on the active workspace, but disappeared
    // without the tiler hearing of it. The tiler did not hide it, so
    // being out of view still counts against it.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.window(2).visible = false;

    // Act -- the rules are loaded again unchanged.
    h.step(Input::Rules(Vec::new()));

    // Assert
    assert_eq!(h.active(0), vec![1]);
    assert!(!showed_anything(&h), "{:?}", h.effects);
}
