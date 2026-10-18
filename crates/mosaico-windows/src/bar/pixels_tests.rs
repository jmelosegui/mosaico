use mosaico_core::Rect;
use mosaico_core::bar::{BarTarget, ItemRegion};

use super::*;

/// A region of the label of workspace 1 at `rect`, `clickable` or not.
fn label(rect: Rect, clickable: bool) -> ItemRegion {
    ItemRegion {
        target: BarTarget::Workspace(0),
        clickable,
        rect,
    }
}

#[test]
fn a_clickable_item_gets_the_least_alpha_and_the_rest_stays_transparent() {
    // Arrange -- a canvas 4 pixels wide and 2 tall, all transparent, with
    // an item over its middle two columns.
    let mut pixels = [0x0000_0000; 8];

    // Act
    leave_no_hole(&mut pixels, 4, &[label(Rect::new(1, 0, 2, 2), true)]);

    // Assert
    let (none, least) = (0x0000_0000, 0x0100_0000);
    assert_eq!(pixels, [none, least, least, none, none, least, least, none]);
}

#[test]
fn an_item_that_takes_no_clicks_is_left_as_painted() {
    // Arrange
    let mut pixels = [0x0000_0000, 0xFF31_3244, 0x0000_0000];

    // Act
    leave_no_hole(&mut pixels, 3, &[label(Rect::new(0, 0, 3, 1), false)]);

    // Assert
    assert_eq!(pixels, [0x0000_0000, 0xFF31_3244, 0x0000_0000]);
}

#[test]
fn a_glyph_edge_left_with_no_alpha_gets_the_least_and_keeps_its_color() {
    // Arrange -- drawn by GDI past the text it measured, on a pill.
    let mut pixels = [0x0031_3266, 0xFF31_3244, 0x7F10_1010];

    // Act
    leave_no_hole(&mut pixels, 3, &[label(Rect::new(0, 0, 3, 1), true)]);

    // Assert -- the pill and a translucent pixel are left as they were.
    assert_eq!(pixels, [0x0131_3266, 0xFF31_3244, 0x7F10_1010]);
}

#[test]
fn the_part_of_an_item_off_the_canvas_is_left_out() {
    // Arrange -- an item from past the top left corner to past the right
    // edge, over the first row only.
    let mut pixels = [0x0000_0000; 6];

    // Act
    leave_no_hole(&mut pixels, 3, &[label(Rect::new(-2, -1, 10, 2), true)]);

    // Assert
    assert_eq!(pixels, [0x0100_0000, 0x0100_0000, 0x0100_0000, 0, 0, 0]);
}

#[test]
fn an_item_or_a_canvas_with_no_area_changes_nothing() {
    // Arrange
    let mut pixels = [0x0000_0000; 4];
    let item = label(Rect::new(1, 0, -1, 2), true);

    // Act
    leave_no_hole(&mut pixels, 2, &[item]);
    leave_no_hole(&mut pixels, 0, &[label(Rect::new(0, 0, 2, 2), true)]);

    // Assert
    assert_eq!(pixels, [0x0000_0000; 4]);
}
