use super::*;

#[test]
fn parse_hex_color() {
    // Assert
    assert_eq!(
        Color::from_hex("#00b4d8"),
        Some(Color {
            r: 0x00,
            g: 0xB4,
            b: 0xD8
        })
    );
    assert_eq!(
        Color::from_hex("2d6a4f"),
        Some(Color {
            r: 0x2D,
            g: 0x6A,
            b: 0x4F
        })
    );
    assert_eq!(Color::from_hex("bad"), None);
}

#[test]
fn a_value_with_a_multibyte_character_is_not_a_color() {
    // #103
    // Arrange -- "#1", e with an acute accent, then "234": six bytes
    // after the '#', since the accented e takes two.
    let value = "#1\u{e9}234";

    // Act
    let parsed = Color::from_hex(value);

    // Assert
    assert_eq!(parsed, None);
}

#[test]
fn a_value_without_a_hash_with_a_multibyte_character_is_not_a_color() {
    // #103
    // Arrange -- the theme keeps a name it does not know as written, so
    // a value without the '#' reaches the parser too.
    let value = "1\u{e9}234";

    // Act
    let parsed = Color::from_hex(value);

    // Assert
    assert_eq!(parsed, None);
}

#[test]
fn a_multibyte_character_in_a_later_pair_is_not_a_color() {
    // #103
    // Arrange -- the accented e takes bytes 3 and 4, so the second pair
    // of digits ends inside it.
    let value = "#123\u{e9}4";

    // Act
    let parsed = Color::from_hex(value);

    // Assert
    assert_eq!(parsed, None);
}

#[test]
fn a_three_byte_character_is_not_a_color() {
    // #103
    // Arrange -- a euro sign takes three bytes.
    let value = "#\u{20ac}123";

    // Act
    let parsed = Color::from_hex(value);

    // Assert
    assert_eq!(parsed, None);
}

#[test]
fn every_spelling_of_a_hex_color_still_parses() {
    // Arrange -- with and without '#', in lower, upper and mixed case,
    // with letters and digits.
    let cases = [
        ("#abcdef", (0xAB, 0xCD, 0xEF)),
        ("abcdef", (0xAB, 0xCD, 0xEF)),
        ("#ABCDEF", (0xAB, 0xCD, 0xEF)),
        ("ABCDEF", (0xAB, 0xCD, 0xEF)),
        ("#aBcDeF", (0xAB, 0xCD, 0xEF)),
        ("1a2B3c", (0x1A, 0x2B, 0x3C)),
    ];

    // Act
    let parsed: Vec<Option<Color>> = cases
        .iter()
        .map(|(value, _)| Color::from_hex(value))
        .collect();

    // Assert
    let expected: Vec<Option<Color>> = cases
        .iter()
        .map(|&(_, (r, g, b))| Some(Color { r, g, b }))
        .collect();
    assert_eq!(parsed, expected);
}

#[test]
fn a_value_after_more_than_one_hash_still_parses() {
    // Arrange -- every leading '#' is taken off.
    let value = "##abcdef";

    // Act
    let parsed = Color::from_hex(value);

    // Assert
    assert_eq!(
        parsed,
        Some(Color {
            r: 0xAB,
            g: 0xCD,
            b: 0xEF
        })
    );
}
