//! Tests of the limits of a split.

use super::*;

const NONE: MinSize = MinSize {
    width: 0,
    height: 0,
};

fn wide(width: i32) -> MinSize {
    MinSize::new(width, 0)
}

#[test]
fn no_minimum_means_no_limits() {
    assert_eq!(limits(LayoutKind::Bsp, &[NONE, NONE], 8, 0, 1904), None);
    assert_eq!(chain_limits(&[NONE, NONE, NONE], true, 8, 1904), None);
}

#[test]
fn the_window_beyond_a_bsp_split_keeps_its_width() {
    // Act
    let found = limits(LayoutKind::Bsp, &[NONE, wide(900)], 8, 0, 1904);

    // Assert -- window 2 keeps 900 pixels and half a gap.
    assert_eq!(found, Some((5, 1904 - 4 - 900)));
}

#[test]
fn the_first_window_of_a_bsp_split_keeps_its_width() {
    assert_eq!(
        limits(LayoutKind::Bsp, &[wide(900), NONE], 8, 0, 1904),
        Some((904, 1904 - 4 - 1))
    );
}

#[test]
fn the_rest_of_a_chain_needs_the_widest_across_and_the_sum_along() {
    // Arrange -- windows 2 and 3 split horizontally in a row: a vertical
    // split of window 2 over 3 does not add widths.
    let mins = [NONE, wide(300), wide(500)];

    // Act
    let first = limits(LayoutKind::Bsp, &mins, 8, 0, 1904);
    let second = limits(LayoutKind::Bsp, &mins, 8, 1, 1064);

    // Assert -- across the other axis it is the widest.
    assert_eq!(first, Some((5, 1904 - 4 - 500)));
    assert_eq!(second, None);
}

#[test]
fn a_chain_split_along_the_axis_again_adds_the_gap() {
    // Arrange -- windows 2, 3, 4: split 2 is horizontal like split 0.
    let mins = [NONE, NONE, NONE, wide(300)];

    // Act
    let found = limits(LayoutKind::Bsp, &mins, 8, 0, 1904);

    // Assert -- window 3 shares the axis with window 4, a gap away.
    assert_eq!(found, Some((5, 1904 - 4 - 308)));
}

#[test]
fn a_vertical_stack_master_keeps_its_width_and_the_stack_its_widest() {
    // Act
    let found = limits(
        LayoutKind::VerticalStack,
        &[wide(700), wide(300), wide(600)],
        8,
        0,
        1904,
    );

    // Assert
    assert_eq!(found, Some((704, 1904 - 4 - 600)));
}

#[test]
fn a_centered_master_leaves_both_columns_their_widest() {
    // Act
    let found = limits(
        LayoutKind::ThreeColumn,
        &[wide(800), wide(300), wide(400), wide(200)],
        8,
        0,
        1904,
    );

    // Assert
    assert_eq!(found, Some((800, 1904 - 16 - 800)));
}

#[test]
fn two_windows_of_three_columns_split_like_a_stack() {
    let found = limits(LayoutKind::ThreeColumn, &[NONE, wide(500)], 8, 0, 1904);
    assert_eq!(found, Some((5, 1904 - 4 - 500)));
}

#[test]
fn a_position_is_kept_within_the_limits() {
    assert_eq!(fit(100, Some((500, 900))), 500);
    assert_eq!(fit(1200, Some((500, 900))), 900);
    assert_eq!(fit(700, Some((500, 900))), 700);
    assert_eq!(fit(700, None), 700);
}

#[test]
fn a_position_stays_where_it_is_when_the_limits_cross() {
    assert_eq!(fit(100, Some((900, 500))), 100);
}
