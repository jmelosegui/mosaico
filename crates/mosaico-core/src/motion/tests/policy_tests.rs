use super::*;

#[path = "policy_apply_tests.rs"]
mod apply;

use apply::applied;

/// A placement of a step that passes every rule.
fn step() -> StepFacts {
    StepFacts {
        enabled: true,
        appears: false,
        behind_monocle: false,
        rebuild: false,
        same_monitor: true,
        windows_moved: 2,
    }
}

#[test]
fn a_placement_that_passes_every_rule_animates() {
    // Act / Assert
    assert!(step_animates(&step()));
    assert!(still_animates(&applied()));
}

#[test]
fn nothing_animates_while_animations_are_off() {
    // Arrange
    let off = StepFacts {
        enabled: false,
        ..step()
    };

    // Act / Assert
    assert!(!step_animates(&off));
}

#[test]
fn a_window_that_appears_in_the_step_snaps_into_place() {
    // Arrange -- shown, taken in, or placed before it was shown.
    let appears = StepFacts {
        appears: true,
        ..step()
    };

    // Act / Assert
    assert!(!step_animates(&appears));
}

#[test]
fn a_window_that_waits_behind_the_monocle_window_snaps_behind_it() {
    // Arrange
    let behind = StepFacts {
        behind_monocle: true,
        ..step()
    };

    // Act / Assert
    assert!(!step_animates(&behind));
}

#[test]
fn a_display_change_the_tiler_starting_or_the_bar_reservation_after_it_snaps_everything() {
    // Arrange -- any of the three: the tiler marks it as a rebuild.
    let rebuild = StepFacts {
        rebuild: true,
        ..step()
    };

    // Act / Assert
    assert!(!step_animates(&rebuild));
}

#[test]
fn a_window_that_moves_to_another_monitor_snaps() {
    // Arrange
    let crossing = StepFacts {
        same_monitor: false,
        ..step()
    };

    // Act / Assert
    assert!(!step_animates(&crossing));
}

#[test]
fn a_step_that_moves_more_than_sixteen_windows_snaps_them_all() {
    // Arrange
    let most = StepFacts {
        windows_moved: MAX_ANIMATED_WINDOWS,
        ..step()
    };
    let more = StepFacts {
        windows_moved: MAX_ANIMATED_WINDOWS + 1,
        ..step()
    };

    // Act / Assert
    assert_eq!(MAX_ANIMATED_WINDOWS, 16);
    assert!(step_animates(&most));
    assert!(!step_animates(&more));
}
