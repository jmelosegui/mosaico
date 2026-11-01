use super::*;

const LINE: [Stroke; 1] = [Stroke::Open(&[(0.0, 12.0), (24.0, 12.0)])];

#[test]
fn every_icon_is_found_by_its_glyph() {
    for icon in BarIcon::ALL {
        assert_eq!(BarIcon::from_glyph(icon.glyph()), Some(icon));
    }
}

#[test]
fn a_glyph_without_a_drawing_has_no_icon() {
    assert_eq!(BarIcon::from_glyph("X"), None);
    assert_eq!(BarIcon::from_glyph(""), None);
}

#[test]
fn a_line_covers_the_pixels_it_runs_through_and_no_others() {
    // Act -- a line across the middle of a 24 pixel square, 2 wide.
    let mask = mask(&LINE, 24, 2.0);

    // Assert
    let at = |x: usize, y: usize| mask[y * 24 + x];
    assert_eq!((at(5, 11), at(5, 12)), (255, 255));
    assert_eq!((at(5, 0), at(5, 23), at(5, 9), at(5, 14)), (0, 0, 0, 0));
}

#[test]
fn the_edge_of_a_line_is_partly_covered() {
    // Act -- 1 wide, so the rows beside it are half a pixel away.
    let mask = mask(&LINE, 24, 1.0);

    // Assert
    assert_eq!(mask[11 * 24 + 5], 128);
}

#[test]
fn a_drawing_is_scaled_to_the_square() {
    // Act -- the same line, in a square half the size.
    let mask = mask(&LINE, 12, 2.0);

    // Assert
    assert_eq!(mask.len(), 144);
    assert_eq!((mask[5 * 12 + 3], mask[6 * 12 + 3]), (255, 255));
    assert_eq!(mask[3], 0);
}

#[test]
fn every_icon_draws_inside_its_square() {
    for icon in BarIcon::ALL {
        // Act
        let mask = mask(icon.strokes(), 16, 1.5);

        // Assert
        assert!(mask.iter().any(|&a| a >= 128), "{icon:?} draws nothing");
        assert!(mask.contains(&0), "{icon:?} fills its square");
    }
}
