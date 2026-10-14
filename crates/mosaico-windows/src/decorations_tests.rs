use mosaico_core::Rect;
use mosaico_core::config::Color;

use super::*;

fn style() -> BorderStyle {
    BorderStyle {
        rect: Rect::new(100, 100, 640, 480),
        color: Color {
            r: 0x11,
            g: 0x22,
            b: 0x33,
        },
        width: 4,
        radius: 8,
    }
}

#[test]
fn a_border_never_drawn_needs_a_bitmap() {
    // Arrange
    let drawn = LastDrawn::default();

    // Act / Assert
    assert!(drawn.needs_render(&style()));
}

#[test]
fn a_border_drawn_again_the_same_way_needs_no_bitmap() {
    // Arrange -- hiding the overlay in between changes nothing here.
    let mut drawn = LastDrawn::default();
    drawn.rendered(&style(), true);

    // Act / Assert
    assert!(!drawn.needs_render(&style()));
}

#[test]
fn a_border_drawn_any_other_way_needs_a_bitmap() {
    // Arrange
    let mut drawn = LastDrawn::default();
    drawn.rendered(&style(), true);
    let mut moved = style();
    moved.rect.x += 1;
    let mut recolored = style();
    recolored.color.r = 0x99;
    let mut wider = style();
    wider.width += 1;
    let mut rounder = style();
    rounder.radius += 1;

    // Act / Assert
    for changed in [moved, recolored, wider, rounder] {
        assert!(drawn.needs_render(&changed), "{changed:?}");
    }
}

#[test]
fn a_bitmap_that_did_not_reach_the_overlay_is_rendered_again() {
    // Arrange
    let mut drawn = LastDrawn::default();
    drawn.rendered(&style(), false);

    // Act / Assert
    assert!(drawn.needs_render(&style()));
}
