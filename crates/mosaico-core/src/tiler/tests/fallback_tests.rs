//! A hiding strategy the platform does not have.

use crate::HidingSupport;
use crate::config::{Config, HidingBehaviour};

use super::*;

/// A platform that cannot cloak windows, with `configured` in the config.
fn cannot_cloak(configured: HidingBehaviour) -> Settings {
    let mut settings = Settings::default();
    settings.layout.hiding = configured;
    settings.hiding_support = HidingSupport::new(|how| match how {
        HidingBehaviour::Cloak => None,
        HidingBehaviour::Hide | HidingBehaviour::Minimize => HidingSupport::default().events(how),
    });
    settings
}

fn hide(window: u64, how: HidingBehaviour) -> Effect {
    Effect::Hide {
        window: wid(window),
        how,
    }
}

fn show(window: u64, how: HidingBehaviour) -> Effect {
    Effect::Show {
        window: wid(window),
        how,
    }
}

#[test]
fn a_strategy_the_platform_lacks_is_swapped_for_one_it_has() {
    // Arrange
    let mut h = Harness::with_settings(cannot_cloak(HidingBehaviour::Cloak), 1, &[(1, 0)]);

    // Act
    let effects = h.action(Action::GoToWorkspace(2)).to_vec();

    // Assert
    assert!(
        effects.contains(&hide(1, HidingBehaviour::Hide)),
        "{effects:?}"
    );
    assert!(!h.desktop.windows[&wid(1)].visible);
    assert!(!h.desktop.windows[&wid(1)].cloaked);
}

#[test]
fn windows_hidden_by_the_fallback_are_shown_the_same_way() {
    // Arrange
    let mut h = Harness::with_settings(cannot_cloak(HidingBehaviour::Cloak), 1, &[(1, 0)]);
    h.action(Action::GoToWorkspace(2));

    // Act
    let effects = h.action(Action::GoToWorkspace(1)).to_vec();

    // Assert
    assert!(
        effects.contains(&show(1, HidingBehaviour::Hide)),
        "{effects:?}"
    );
    assert_eq!(h.active(0), vec![1]);
}

#[test]
fn reloading_a_strategy_the_platform_lacks_falls_back_too() {
    // Arrange -- minimizing is available, cloaking is not.
    let mut h = Harness::with_settings(cannot_cloak(HidingBehaviour::Minimize), 1, &[(1, 0)]);
    let mut config = Config::default();
    config.layout.hiding = HidingBehaviour::Cloak;
    h.step(Input::Config(Box::new(config)));

    // Act
    let effects = h.action(Action::GoToWorkspace(2)).to_vec();

    // Assert
    assert!(
        effects.contains(&hide(1, HidingBehaviour::Hide)),
        "{effects:?}"
    );
}

#[test]
fn reloading_an_unchanged_strategy_keeps_its_fallback() {
    // Arrange -- the config asks for cloaking at startup and still does.
    let mut h = Harness::with_settings(cannot_cloak(HidingBehaviour::Cloak), 1, &[(1, 0)]);
    let mut config = Config::default();
    config.layout.hiding = HidingBehaviour::Cloak;
    config.layout.gap = 16;
    h.step(Input::Config(Box::new(config)));

    // Act
    let effects = h.action(Action::GoToWorkspace(2)).to_vec();

    // Assert
    assert!(
        effects.contains(&hide(1, HidingBehaviour::Hide)),
        "{effects:?}"
    );
}

#[test]
fn the_next_daemon_shows_a_window_the_way_the_fallback_hid_it() {
    // Arrange
    let mut h = Harness::with_settings(cannot_cloak(HidingBehaviour::Cloak), 1, &[(1, 0)]);

    // Act
    h.action(Action::GoToWorkspace(2));

    // Assert
    let desktop = &h.desktop;
    assert_eq!(
        h.hidden.restore(|w| desktop.identity(w)),
        [show(1, HidingBehaviour::Hide)]
    );
}

#[test]
fn hide_reports_the_fallback_causes_are_taken_for_the_tilers_own() {
    // Arrange -- hiding reports the window hidden. The window must be
    // really hidden: a hide report for a window still shown is ignored
    // whether or not it is an echo.
    let settings = cannot_cloak(HidingBehaviour::Cloak);
    let mut h = Harness::with_settings(settings, 1, &[(1, 0)]).through_platform();
    h.action(Action::GoToWorkspace(2));
    assert!(!h.desktop.windows[&wid(1)].visible);

    // Act
    let effects = h.event(WindowEvent::Hidden { window: wid(1) }).to_vec();

    // Assert
    assert!(effects.is_empty(), "{effects:?}");
    assert!(h.workspace(0, 0).contains(wid(1)));
}
