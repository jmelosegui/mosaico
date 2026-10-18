//! The heights of the vertical stack's stack, kept in step with the
//! windows: a window that joins a resized stack gets an equal share and the
//! others shrink in proportion, and one that leaves gives its share back in
//! proportion. Windows given back what they owned have their heights back,
//! and a retile forgets them. The three columns are in
//! `split_column_chain_tests.rs`.

use crate::LayoutKind::{self, ThreeColumn, VerticalStack};

use super::super::split_support::unmoved;
use super::SplitRatios;

/// `count` windows, the column window `window` of `kind` is in at `shares`.
pub(super) fn resized(
    kind: LayoutKind,
    count: usize,
    window: usize,
    shares: &[f64],
) -> SplitRatios {
    let mut ratios = unmoved(count);
    ratios.set_heights(kind, window, shares);
    ratios
}

/// The shares of the column window `window` of `kind` is in, to a
/// millionth, and none for a column nobody resized.
pub(super) fn heights(ratios: &SplitRatios, kind: LayoutKind, window: usize) -> Vec<f64> {
    let shares = ratios.heights(kind, window).unwrap_or_default();
    shares
        .into_iter()
        .map(|s| (s * 1e6).round() / 1e6)
        .collect()
}

#[test]
fn a_window_that_joins_a_resized_stack_gets_an_equal_share_and_the_others_shrink() {
    // Arrange -- the master and four windows down the stack, resized, and
    // the same nobody resized.
    let mut ratios = resized(VerticalStack, 5, 1, &[0.4, 0.3, 0.2, 0.1]);
    let mut equal = unmoved(5);

    // Act -- a window goes in third, second of the stack.
    ratios.insert(2, 5);
    equal.insert(2, 5);

    // Assert -- a fifth, and four fifths of what each had.
    assert_eq!(
        heights(&ratios, VerticalStack, 1),
        [0.32, 0.2, 0.24, 0.16, 0.08]
    );
    assert_eq!(equal, unmoved(6));
}

#[test]
fn a_window_that_leaves_a_resized_stack_gives_its_share_back_in_proportion() {
    // Arrange
    let mut ratios = resized(VerticalStack, 5, 1, &[0.4, 0.3, 0.2, 0.1]);

    // Act -- the second of the stack leaves.
    ratios.remove(2, 5);

    // Assert
    let shares = heights(&ratios, VerticalStack, 1);
    assert_eq!(shares, [0.571429, 0.285714, 0.142857]);
}

#[test]
fn the_master_moves_to_the_top_of_the_stack_and_the_top_of_the_stack_up() {
    // Arrange
    let stack = || {
        let mut ratios = resized(VerticalStack, 5, 1, &[0.4, 0.3, 0.2, 0.1]);
        ratios.set(VerticalStack, 0, 0.3);
        ratios
    };

    // Act -- a window takes the master's place, and the master leaves.
    let mut joined = stack();
    joined.insert(0, 5);
    let mut left = stack();
    left.remove(0, 5);

    // Assert -- the old master joins at the top, the top leaves, and the
    // master's ratio stays.
    assert_eq!(
        heights(&joined, VerticalStack, 1),
        [0.2, 0.32, 0.24, 0.16, 0.08]
    );
    assert_eq!(heights(&left, VerticalStack, 1), [0.5, 0.333333, 0.166667]);
    assert_eq!(joined.ratio(VerticalStack, 0), Some(0.3));
}

#[test]
fn windows_given_back_what_they_owned_have_their_heights_back_where_they_still_can() {
    // Arrange -- six windows, the stack and both columns resized: windows
    // 1, 3 and 5 on the left, and 2 and 4 on the right.
    let mut had = resized(VerticalStack, 6, 1, &[0.3, 0.25, 0.2, 0.15, 0.1]);
    had.set_heights(ThreeColumn, 1, &[0.5, 0.3, 0.2]);
    had.set_heights(ThreeColumn, 2, &[0.6, 0.4]);
    let owned: Vec<_> = (0..6)
        .map(|index| (index, index, had.owned_by(index)))
        .collect();

    // Act -- all back where they were, window 1 back as the master, and
    // window 1 back in the right column.
    let mut whole = unmoved(6);
    whole.give(&owned, 6);
    let mut master = unmoved(6);
    master.give(&[(0, 1, owned[1].2)], 6);
    let mut across = unmoved(6);
    across.give(&[(2, 1, owned[1].2)], 6);

    // Assert -- the master brings no share of the stack, and the window
    // in the other column none of a column.
    let stack = heights(&whole, VerticalStack, 1);
    assert_eq!(stack, [0.3, 0.25, 0.2, 0.15, 0.1]);
    let both = [1, 2].map(|window| heights(&whole, ThreeColumn, window));
    assert_eq!(both, [vec![0.5, 0.3, 0.2], vec![0.6, 0.4]]);
    assert_eq!(master, unmoved(6));
    let columns = [1, 2].map(|window| heights(&across, ThreeColumn, window));
    assert_eq!(columns, [Vec::<f64>::new(), Vec::new()]);
}

#[test]
fn forgetting_the_heights_shares_every_stack_equally_again() {
    // Arrange
    let mut ratios = resized(VerticalStack, 5, 1, &[0.4, 0.3, 0.2, 0.1]);
    ratios.set_heights(ThreeColumn, 2, &[0.6, 0.4]);

    // Act -- as a retile does.
    ratios.forget();

    // Assert
    assert_eq!(ratios, unmoved(5));
}
