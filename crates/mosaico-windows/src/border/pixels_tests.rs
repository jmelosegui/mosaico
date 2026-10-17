use super::*;

#[test]
fn rounded_rect_zero_radius_is_full_rect() {
    // Arrange / Act / Assert
    assert!(in_rounded_rect(0, 0, 10, 10, 0));
    assert!(in_rounded_rect(9, 9, 10, 10, 0));
}

#[test]
fn rounded_rect_outside_is_false() {
    // Arrange / Act / Assert
    assert!(!in_rounded_rect(-1, 0, 10, 10, 2));
    assert!(!in_rounded_rect(0, -1, 10, 10, 2));
    assert!(!in_rounded_rect(10, 0, 10, 10, 2));
    assert!(!in_rounded_rect(0, 10, 10, 10, 2));
}

#[test]
fn rounded_rect_center_is_inside() {
    // Arrange / Act / Assert
    assert!(in_rounded_rect(5, 5, 10, 10, 4));
}

#[test]
fn rounded_rect_corner_pixel_outside_circle() {
    // Arrange -- top-left corner (0,0) with radius 5
    // Distance from center (5,5): sqrt(25+25) > 5

    // Act / Assert
    assert!(!in_rounded_rect(0, 0, 10, 10, 5));
}

#[test]
fn rounded_rect_edge_not_in_corner_is_inside() {
    // Arrange -- pixel on the top edge but not in a corner zone

    // Act / Assert
    assert!(in_rounded_rect(5, 0, 10, 10, 3));
}

/// A color whose channels scale apart.
const COLOR: Color = Color {
    r: 0xFF,
    g: 0x80,
    b: 0x00,
};

#[test]
fn a_premultiplied_pixel_scales_each_channel_by_its_alpha() {
    // Act
    let opaque = premultiplied(COLOR, u8::MAX);
    let quarter = premultiplied(COLOR, 64);
    let clear = premultiplied(COLOR, 0);

    // Assert
    assert_eq!(opaque, 0xFFFF_8000);
    assert_eq!(quarter, 0x4040_2000);
    assert_eq!(clear, 0);
}

#[test]
fn a_border_is_its_stroke_around_a_clear_middle() {
    // Arrange -- 6 by 6, a stroke 2 wide, square corners.
    let mut buf = vec![7u32; 36];

    // Act
    paint(&mut buf, 6, 6, 2, 0, 1, 0);

    // Assert -- the edge rows and columns are the stroke, the middle clear.
    let at = |x: usize, y: usize| buf[y * 6 + x];
    assert_eq!((at(0, 0), at(1, 3), at(5, 5), at(3, 4)), (1, 1, 1, 1));
    assert_eq!((at(2, 2), at(3, 3)), (0, 0));
}

#[test]
fn a_preview_fills_inside_its_stroke_and_leaves_its_corners_clear() {
    // Arrange -- 20 by 20, a stroke 2 wide, corners rounded by 5.
    let mut buf = vec![7u32; 400];

    // Act
    paint(&mut buf, 20, 20, 2, 5, 1, 2);

    // Assert
    let at = |x: usize, y: usize| buf[y * 20 + x];
    assert_eq!(at(0, 0), 0, "outside the rounded corner");
    assert_eq!(at(10, 0), 1, "on the stroke");
    assert_eq!(at(10, 10), 2, "inside");
}
