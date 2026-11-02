use super::*;

use crate::LayoutKind;

use super::super::split_edges::border;
use super::super::split_support::{GAP, KINDS, after, slots, unmoved};

/// How wide or tall `rect` is along `axis`.
fn size(rect: Rect, axis: Axis) -> i32 {
    match axis {
        Axis::Width => rect.width,
        Axis::Height => rect.height,
    }
}

/// The edges of a window along `axis`.
fn sides(axis: Axis) -> [Side; 2] {
    match axis {
        Axis::Width => [Side::Left, Side::Right],
        Axis::Height => [Side::Top, Side::Bottom],
    }
}

#[test]
fn a_step_grows_or_shrinks_every_tile_that_borders_a_split_by_the_step() {
    for kind in KINDS {
        for count in 2..=5 {
            let ratios = unmoved(count);
            let before = slots(kind, count, &ratios);
            for index in 0..count {
                for axis in [Axis::Width, Axis::Height] {
                    for (grow, sign) in [(true, 1), (false, -1)] {
                        // Arrange
                        let case = format!("{kind:?}, window {index} of {count}, {axis:?}");
                        let borders = sides(axis)
                            .into_iter()
                            .any(|side| border(kind, count, index, side).is_some());

                        // Act
                        let tiles = Tiles::new(kind, GAP, &before, &ratios);
                        let stepped = stepped_splits(tiles, index, axis, grow, 40);

                        // Assert -- to the pixel, once laid out again.
                        assert_eq!(stepped.is_some(), borders, "{case}");
                        if stepped.is_some() {
                            let laid_out = after(kind, count, &stepped)[index];
                            let by = size(laid_out, axis) - size(before[index], axis);
                            assert_eq!(by, sign * 40, "{case}, grow {grow}");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn the_last_window_of_a_row_grows_towards_the_others() {
    // Arrange -- the right one of two side by side.
    let ratios = unmoved(2);
    let before = slots(LayoutKind::Bsp, 2, &ratios);

    // Act
    let tiles = Tiles::new(LayoutKind::Bsp, GAP, &before, &ratios);
    let stepped = stepped_splits(tiles, 1, Axis::Width, true, 40);

    // Assert -- its right edge stays at the edge of the area.
    let laid_out = after(LayoutKind::Bsp, 2, &stepped);
    assert_eq!(laid_out[1].x, before[1].x - 40);
    let right = |r: Rect| r.x + r.width;
    assert_eq!(right(laid_out[1]), right(before[1]));
    assert_eq!(laid_out[0].width, before[0].width - 40);
}

#[test]
fn the_master_of_three_columns_grows_by_a_step_and_stays_centered() {
    // Arrange
    let ratios = unmoved(3);
    let before = slots(LayoutKind::ThreeColumn, 3, &ratios);

    // Act
    let tiles = Tiles::new(LayoutKind::ThreeColumn, GAP, &before, &ratios);
    let stepped = stepped_splits(tiles, 0, Axis::Width, true, 40);

    // Assert
    let laid_out = after(LayoutKind::ThreeColumn, 3, &stepped);
    assert_eq!(laid_out[0].width, before[0].width + 40);
    assert_eq!(laid_out[0].center_x(), before[0].center_x());
}

#[test]
fn a_tile_that_borders_no_split_along_the_axis_moves_none() {
    // Arrange -- the only window, and the height of the master of a stack.
    let one = unmoved(1);
    let alone = slots(LayoutKind::Bsp, 1, &one);
    let two = unmoved(2);
    let stack = slots(LayoutKind::VerticalStack, 2, &two);

    // Act
    let only = Tiles::new(LayoutKind::Bsp, GAP, &alone, &one);
    let master = Tiles::new(LayoutKind::VerticalStack, GAP, &stack, &two);

    // Assert
    assert_eq!(stepped_splits(only, 0, Axis::Width, true, 40), None);
    assert_eq!(stepped_splits(master, 0, Axis::Height, true, 40), None);
}
