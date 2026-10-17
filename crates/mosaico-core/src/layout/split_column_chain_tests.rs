//! The heights of the columns of the three columns, kept in step with the
//! windows: the windows after the master alternate between the left column
//! and the right one, so one that comes or goes before the last of them
//! sends both columns back to equal heights, and one that joins after the
//! last, or the last that leaves, changes only its own column.

use crate::LayoutKind::ThreeColumn;

use super::SplitRatios;
use super::tests::{heights, resized};

/// Seven windows: the master, windows 1, 3 and 5 on the left and 2, 4 and
/// 6 on the right, both columns resized.
fn columns() -> SplitRatios {
    let mut ratios = resized(ThreeColumn, 7, 1, &[0.5, 0.3, 0.2]);
    ratios.set_heights(ThreeColumn, 2, &[0.6, 0.3, 0.1]);
    ratios
}

#[test]
fn a_window_that_comes_or_goes_before_the_last_side_window_evens_both_columns() {
    // Act -- a window goes in fourth, window 3 leaves, window 5, the last
    // but one, leaves, a window takes the master's place, and the master
    // leaves.
    let changes: [fn(&mut SplitRatios); 5] = [
        |r| r.insert(3, 7),
        |r| r.remove(3, 7),
        |r| r.remove(5, 7),
        |r| r.insert(0, 7),
        |r| r.remove(0, 7),
    ];

    // Assert -- the side windows after it changed column.
    for change in changes {
        let mut ratios = columns();
        change(&mut ratios);
        let both = [1, 2].map(|window| heights(&ratios, ThreeColumn, window));
        assert_eq!(both, [Vec::<f64>::new(), Vec::new()]);
    }
}

#[test]
fn a_window_that_joins_after_the_last_side_window_or_is_the_last_to_leave_changes_its_column() {
    // Act -- a window joins at the end, in the left column, and the last
    // window, in the right one, leaves.
    let mut joined = columns();
    joined.insert(7, 7);
    let mut left = columns();
    left.remove(6, 7);

    // Assert -- the other column keeps its heights.
    let both = |ratios: &SplitRatios| [1, 2].map(|w| heights(ratios, ThreeColumn, w));
    let (wide, narrow) = (vec![0.5, 0.3, 0.2], vec![0.6, 0.3, 0.1]);
    assert_eq!(both(&joined), [vec![0.375, 0.225, 0.15, 0.25], narrow]);
    assert_eq!(both(&left), [wide, vec![0.666667, 0.333333]]);
}
