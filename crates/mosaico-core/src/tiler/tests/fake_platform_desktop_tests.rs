//! What the fake platform answers about the desktop, which the tests
//! rely on: the system's animation setting.

use super::*;

#[test]
fn animations_are_allowed_until_the_user_turns_them_off() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0)]);
    let by_default = platform.animations_allowed();

    // Act
    platform.allow_animations(false);

    // Assert -- read again each time it is asked.
    assert!(by_default);
    assert!(!platform.animations_allowed());
}
