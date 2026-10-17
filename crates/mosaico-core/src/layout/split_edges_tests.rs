use super::*;

use super::super::split_support::{KINDS, SIDES};

/// The split each edge of each of `count` windows borders in `kind`, as
/// `[left, top, right, bottom]`.
fn bordered(kind: LayoutKind, count: usize) -> Vec<[Option<usize>; 4]> {
    let split = |index, side| border(kind, count, index, side).map(|(split, _)| split);
    (0..count)
        .map(|index| SIDES.map(|side| split(index, side)))
        .collect()
}

#[test]
fn in_bsp_a_window_borders_its_own_split_and_the_two_it_lies_beyond() {
    // Act
    let four = bordered(LayoutKind::Bsp, 4);

    // Assert -- left, top, right, bottom; the last window has no split of
    // its own.
    let expected = [
        [None, None, Some(0), None],
        [Some(0), None, None, Some(1)],
        [Some(0), Some(1), Some(2), None],
        [Some(2), Some(1), None, None],
    ];
    assert_eq!(four, expected);
}

#[test]
fn in_the_vertical_stack_only_the_edge_of_the_master_borders_a_split() {
    // Act
    let four = bordered(LayoutKind::VerticalStack, 4);

    // Assert -- the edges between windows of the stack border none.
    let stacked = [Some(0), None, None, None];
    let expected = [[None, None, Some(0), None], stacked, stacked, stacked];
    assert_eq!(four, expected);
}

#[test]
fn in_three_columns_the_master_and_the_inner_edges_of_the_columns_border_it() {
    // Act -- the windows after the master go left, right, left, right.
    let five = bordered(LayoutKind::ThreeColumn, 5);
    let two = bordered(LayoutKind::ThreeColumn, 2);

    // Assert -- two windows split as in the vertical stack.
    let (left, right) = ([None, None, Some(0), None], [Some(0), None, None, None]);
    assert_eq!(
        five,
        [[Some(0), None, Some(0), None], left, right, left, right]
    );
    assert_eq!(two, [left, right]);
}

#[test]
fn a_window_alone_or_not_there_borders_no_split() {
    for kind in KINDS {
        // Act
        let alone = bordered(kind, 1);
        let missing = SIDES.map(|side| border(kind, 2, 2, side));

        // Assert
        assert_eq!(alone, [[None; 4]], "{kind:?}");
        assert_eq!(missing, [None; 4], "{kind:?}");
    }
}
