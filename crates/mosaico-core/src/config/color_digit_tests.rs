//! Which characters count as the hex digits of a color.

use super::*;

#[test]
fn a_sign_is_not_a_hex_digit() {
    // Arrange -- the integer parser takes a leading '+', so this used to
    // read as #0f0f0f.
    let value = "#+f+f+f";

    // Act
    let parsed = Color::from_hex(value);

    // Assert
    assert_eq!(parsed, None);
}
