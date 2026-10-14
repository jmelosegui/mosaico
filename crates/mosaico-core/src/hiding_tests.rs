use crate::config::HidingBehaviour;

use super::{HidingEvents, HidingSupport};

/// Hiding reports the window hidden, and showing it again reports
/// nothing.
const HIDE_ONLY: HidingEvents = HidingEvents {
    reports_hidden: true,
    reports_restored: false,
};

/// A platform that has only the ways in `ways`.
fn only(ways: &[HidingBehaviour]) -> HidingSupport {
    HidingSupport::new(|how| ways.contains(&how).then_some(HIDE_ONLY))
}

#[test]
fn a_way_the_platform_has_is_used_as_configured() {
    // Arrange
    let support = HidingSupport::default();

    // Act / Assert
    for how in [
        HidingBehaviour::Cloak,
        HidingBehaviour::Hide,
        HidingBehaviour::Minimize,
    ] {
        assert_eq!(support.choose(how), how);
    }
}

#[test]
fn cloaking_falls_back_to_hiding_first() {
    // Arrange
    let support = only(&[HidingBehaviour::Hide, HidingBehaviour::Minimize]);

    // Act
    let used = support.choose(HidingBehaviour::Cloak);

    // Assert
    assert_eq!(used, HidingBehaviour::Hide);
}

#[test]
fn cloaking_and_hiding_fall_back_to_minimizing_when_it_is_all_there_is() {
    // Arrange
    let support = only(&[HidingBehaviour::Minimize]);

    // Act / Assert
    assert_eq!(
        support.choose(HidingBehaviour::Cloak),
        HidingBehaviour::Minimize
    );
    assert_eq!(
        support.choose(HidingBehaviour::Hide),
        HidingBehaviour::Minimize
    );
}

#[test]
fn hiding_falls_back_to_cloaking_which_is_just_as_invisible() {
    // Arrange
    let support = only(&[HidingBehaviour::Cloak, HidingBehaviour::Minimize]);

    // Act
    let used = support.choose(HidingBehaviour::Hide);

    // Assert
    assert_eq!(used, HidingBehaviour::Cloak);
}

#[test]
fn minimizing_falls_back_to_cloaking_which_keeps_the_taskbar_entry_too() {
    // Arrange
    let support = only(&[HidingBehaviour::Cloak, HidingBehaviour::Hide]);

    // Act
    let used = support.choose(HidingBehaviour::Minimize);

    // Assert
    assert_eq!(used, HidingBehaviour::Cloak);
}

#[test]
fn a_platform_that_cannot_hide_windows_keeps_the_configured_way() {
    // Arrange
    let support = only(&[]);

    // Act
    let used = support.choose(HidingBehaviour::Minimize);

    // Assert
    assert_eq!(used, HidingBehaviour::Minimize);
}

#[test]
fn the_support_answers_what_the_platform_said() {
    // Arrange -- a platform that cannot cloak.
    let support = HidingSupport::new(|how| match how {
        HidingBehaviour::Cloak => None,
        HidingBehaviour::Hide | HidingBehaviour::Minimize => Some(HIDE_ONLY),
    });

    // Act / Assert
    assert_eq!(support.events(HidingBehaviour::Cloak), None);
    assert_eq!(support.events(HidingBehaviour::Hide), Some(HIDE_ONLY));
    assert_eq!(support.events(HidingBehaviour::Minimize), Some(HIDE_ONLY));
}

#[test]
fn the_platform_is_asked_about_every_way_of_hiding() {
    // Arrange
    let mut asked = Vec::new();

    // Act
    HidingSupport::new(|how| {
        asked.push(how);
        None
    });

    // Assert
    assert_eq!(
        asked,
        [
            HidingBehaviour::Cloak,
            HidingBehaviour::Hide,
            HidingBehaviour::Minimize
        ]
    );
}

#[test]
fn by_default_every_way_reports_what_it_usually_does() {
    // Arrange
    let support = HidingSupport::default();

    // Act / Assert
    assert_eq!(
        support.events(HidingBehaviour::Cloak),
        Some(HidingEvents::default()),
        "cloaking reports nothing"
    );
    assert_eq!(support.events(HidingBehaviour::Hide), Some(HIDE_ONLY));
    assert_eq!(
        support.events(HidingBehaviour::Minimize),
        Some(HidingEvents {
            reports_hidden: true,
            reports_restored: true,
        }),
        "a minimized window reports being restored when shown again"
    );
}
