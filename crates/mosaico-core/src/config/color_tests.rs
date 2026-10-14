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
