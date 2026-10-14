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
