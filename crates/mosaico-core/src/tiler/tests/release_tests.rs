use std::time::{Duration, Instant};

use super::*;

#[test]
fn held_effects_are_due_once_the_platform_has_waited_quietly() {
    // Arrange
    let t0 = Instant::now();

    // Act
    let due = release_at(t0, t0);

    // Assert
    assert_eq!(due, t0 + QUIET);
}

#[test]
fn the_max_hold_cuts_a_late_quiet_window_short() {
    // Arrange -- #74: input has kept ending the platform's waits since it
    // first waited with the effects held, 90 ms ago.
    let t0 = Instant::now();

    // Act
    let due = release_at(t0, t0 + Duration::from_millis(90));

    // Assert
    assert_eq!(due, t0 + MAX_HOLD);
}
