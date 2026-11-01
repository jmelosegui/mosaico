use super::*;

const SIZE: usize = 32;

fn icon(color: [u8; 4]) -> Vec<[u8; 4]> {
    vec![color; SIZE * SIZE]
}

fn at(pixels: &[[u8; 4]], x: usize, y: usize) -> [u8; 4] {
    pixels[y * SIZE + x]
}

#[test]
fn the_icon_away_from_the_badge_is_faded_and_grayed() {
    // Arrange
    let mut pixels = icon([0x40, 0x80, 0xF0, 0xFF]);

    // Act
    paint_paused(&mut pixels, SIZE);

    // Assert -- more transparent, and the channels closer together.
    let [r, g, b, a] = at(&pixels, 2, 2);
    assert!(a < 0xA0, "alpha {a}");
    assert!(b - r < (0xF0 - 0x40) / 2, "{r} {g} {b}");
}

#[test]
fn the_badge_is_red_and_opaque_with_white_bars() {
    // Arrange -- even over a transparent corner.
    let mut pixels = icon([0, 0, 0, 0]);
    let side = SIZE as f32;
    let center = (side - side * 0.3) as usize;

    // Act
    paint_paused(&mut pixels, SIZE);

    // Assert -- red between the bars and near the rim, white on a bar.
    let [r, g, b, a] = at(&pixels, center, center);
    assert_eq!(
        [r, g, b, a],
        [PAUSED_BADGE[0], PAUSED_BADGE[1], PAUSED_BADGE[2], 0xFF]
    );
    assert_eq!(at(&pixels, center + 2, center), [0xFF, 0xFF, 0xFF, 0xFF]);
    assert_eq!(at(&pixels, center, center + 7)[3], 0xFF);
}

#[test]
fn a_transparent_pixel_away_from_the_badge_stays_transparent() {
    // Arrange
    let mut pixels = icon([0, 0, 0, 0]);

    // Act
    paint_paused(&mut pixels, SIZE);

    // Assert
    assert_eq!(at(&pixels, 0, 0)[3], 0);
    assert_eq!(at(&pixels, SIZE - 1, 0)[3], 0);
}

#[test]
fn pixels_of_another_size_are_left_alone() {
    // Arrange
    let mut pixels = icon([1, 2, 3, 4]);

    // Act
    paint_paused(&mut pixels, SIZE + 1);

    // Assert
    assert_eq!(pixels, icon([1, 2, 3, 4]));
}
