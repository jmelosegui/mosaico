use crate::config::HidingBehaviour;

use super::{HidingEvents, HidingSupport};

/// Hiding reports the window hidden, and showing it again reports
/// nothing.
const HIDE_ONLY: HidingEvents = HidingEvents {
    reports_hidden: true,
    reports_restored: false,
};

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
