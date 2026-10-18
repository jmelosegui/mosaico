//! What windows coming and going do to the heights of the columns of the
//! three columns: a window that closes before the last side window takes
//! every side window after it to the other column, and sends both columns
//! back to equal heights, while the last one closing, or one opening after
//! it, changes only its own column.

use crate::LayoutKind;
use crate::WindowEvent;
use crate::id::wid;
use crate::tiler::tests::Harness;

use super::split_tests::rects;
use super::stack_tests::{heights, in_layout, resize};

/// Windows 1 to 7 in three columns, 2, 4 and 6 on the left and 3, 5 and 7
/// on the right, both columns resized.
fn columns() -> Harness {
    let mut h = in_layout(LayoutKind::ThreeColumn, 7);
    resize(&mut h, 1, &[0.5, 0.3, 0.2]);
    resize(&mut h, 2, &[0.6, 0.3, 0.1]);
    h
}

#[test]
fn a_window_that_closes_before_the_last_in_three_columns_evens_both_columns() {
    // Arrange
    let mut h = columns();
    let equal = rects(&in_layout(LayoutKind::ThreeColumn, 6), &[1, 2, 3, 4, 5, 6]);

    // Act -- window 4, second of the left column, closes, which takes
    // windows 5 to 7 to the other column.
    h.desktop.windows.remove(&wid(4));
    h.event(WindowEvent::Destroyed { window: wid(4) });

    // Assert -- both columns shared equally, as nobody resized them.
    let both = [1, 2].map(|index| heights(&h, LayoutKind::ThreeColumn, index));
    assert_eq!(both, [Vec::<f64>::new(), Vec::new()]);
    assert_eq!(rects(&h, &[1, 2, 3, 5, 6, 7]), equal);
}

#[test]
fn the_last_window_of_three_columns_changes_only_its_column_as_it_comes_or_goes() {
    // Arrange
    let (mut closed, mut opened) = (columns(), columns());

    // Act -- window 7, the last, closes, and window 8 opens after it, in
    // the left column.
    closed.desktop.windows.remove(&wid(7));
    closed.event(WindowEvent::Destroyed { window: wid(7) });
    opened.open(8, 0);

    // Assert -- the other column keeps its heights.
    let both = |h: &Harness| [1, 2].map(|index| heights(h, LayoutKind::ThreeColumn, index));
    let (left, right) = (vec![0.5, 0.3, 0.2], vec![0.6, 0.3, 0.1]);
    assert_eq!(both(&closed), [left, vec![0.666667, 0.333333]]);
    assert_eq!(both(&opened), [vec![0.375, 0.225, 0.15, 0.25], right]);
}
