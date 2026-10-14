use std::time::Instant;

use super::*;

#[test]
fn held_effects_are_due_once_the_platform_has_waited_quietly() {
    // Arrange
    let t0 = Instant::now();

    // Act
    let due = release_at(t0);

    // Assert
    assert_eq!(due, t0 + QUIET);
}
