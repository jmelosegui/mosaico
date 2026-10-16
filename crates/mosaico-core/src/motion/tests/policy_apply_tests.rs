use crate::Rect;

use super::super::{ApplyFacts, MIN_TRAVEL, still_animates};

/// A placement that passes every rule when it is applied: a window
/// moving across half of a 1600 px screen.
pub(super) fn applied() -> ApplyFacts {
    ApplyFacts {
        follow_system: true,
        system_allows: true,
        minimized: false,
        maximized: false,
        frames_supported: true,
        from: Rect::new(0, 0, 800, 900),
        to: Rect::new(800, 0, 800, 900),
    }
}

#[test]
fn nothing_animates_while_the_system_turns_animations_off() {
    // Arrange
    let disallowed = ApplyFacts {
        system_allows: false,
        ..applied()
    };
    let ignored = ApplyFacts {
        follow_system: false,
        ..disallowed
    };

    // Act / Assert -- unless the config does not follow the system.
    assert!(!still_animates(&disallowed));
    assert!(still_animates(&ignored));
}

#[test]
fn a_minimized_or_maximized_window_snaps() {
    // Arrange
    let minimized = ApplyFacts {
        minimized: true,
        ..applied()
    };
    let maximized = ApplyFacts {
        maximized: true,
        ..applied()
    };

    // Act / Assert
    assert!(!still_animates(&minimized));
    assert!(!still_animates(&maximized));
}

#[test]
fn a_window_that_moves_or_resizes_by_four_pixels_or_less_snaps() {
    // Arrange
    let from = Rect::new(100, 100, 800, 600);
    let to = |x, y, width, height| ApplyFacts {
        from,
        to: Rect::new(x, y, width, height),
        ..applied()
    };

    // Act / Assert -- a move, a resize from the right and bottom, and
    // one from the left, which moves the left edge alone.
    assert_eq!(MIN_TRAVEL, 4);
    assert!(!still_animates(&to(100, 100, 800, 600)));
    assert!(!still_animates(&to(104, 96, 800, 600)));
    assert!(!still_animates(&to(100, 100, 804, 596)));
    assert!(!still_animates(&to(96, 100, 804, 600)));
    assert!(still_animates(&to(105, 100, 800, 600)));
    assert!(still_animates(&to(100, 100, 800, 605)));
    assert!(still_animates(&to(95, 100, 805, 600)));
}

#[test]
fn nothing_animates_on_a_platform_without_frames() {
    // Arrange
    let unsupported = ApplyFacts {
        frames_supported: false,
        ..applied()
    };

    // Act / Assert
    assert!(!still_animates(&unsupported));
}
