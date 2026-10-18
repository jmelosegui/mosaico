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
fn in_the_vertical_stack_the_master_and_the_edges_between_windows_of_the_stack_border_splits() {
    // Act
    let four = bordered(LayoutKind::VerticalStack, 4);

    // Assert -- left, top, right, bottom: the stack's windows border the
    // master's split with their left edge, the split above them with their
    // top edge and their own with their bottom edge, but for the first and
    // the last of the stack.
    let expected = [
        [None, None, Some(0), None],
        [Some(0), None, None, Some(1)],
        [Some(0), Some(1), None, Some(2)],
        [Some(0), Some(2), None, None],
    ];
    assert_eq!(four, expected);
}

#[test]
fn in_three_columns_the_master_the_inner_edges_and_those_down_a_column_border_splits() {
    // Act -- the windows after the master go left, right, left, right.
    let five = bordered(LayoutKind::ThreeColumn, 5);
    let two = bordered(LayoutKind::ThreeColumn, 2);

    // Assert -- windows 1 and 3 share the left column, and 2 and 4 the
    // right one; two windows split as in the vertical stack.
    let expected = [
        [Some(0), None, Some(0), None],
        [None, None, Some(0), Some(1)],
        [Some(0), None, None, Some(2)],
        [None, Some(1), Some(0), None],
        [Some(0), Some(2), None, None],
    ];
    assert_eq!(five, expected);
    assert_eq!(
        two,
        [[None, None, Some(0), None], [Some(0), None, None, None]]
    );
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
