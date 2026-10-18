//! Tests of fitting a column to its minimum heights.

use super::*;

fn column(heights: &[i32], gap: i32) -> Vec<Rect> {
    let mut y = 8;
    heights
        .iter()
        .map(|&h| {
            let rect = Rect::new(0, y, 500, h);
            y += h + gap;
            rect
        })
        .collect()
}

fn tall(heights: &[i32]) -> Vec<MinSize> {
    heights.iter().map(|&h| MinSize::new(0, h)).collect()
}

fn heights(slots: &[Rect]) -> Vec<i32> {
    slots.iter().map(|s| s.height).collect()
}

#[test]
fn a_column_with_no_window_below_its_minimum_is_left_alone() {
    // Arrange
    let mut slots = column(&[300, 300, 300], 8);
    let before = slots.clone();

    // Act
    fit_column(&mut slots, &tall(&[200, 0, 300]), 8);

    // Assert
    assert_eq!(slots, before);
}

#[test]
fn a_window_below_its_minimum_takes_the_height_from_the_others() {
    // Arrange
    let mut slots = column(&[300, 300, 300], 8);

    // Act
    fit_column(&mut slots, &tall(&[500, 0, 0]), 8);

    // Assert -- the other two give 100 each, the column's ends stay.
    assert_eq!(heights(&slots), vec![500, 200, 200]);
    assert_eq!(slots[0].y, 8);
    assert_eq!(slots[2].y + slots[2].height, 8 + 900 + 16);
}

#[test]
fn a_window_without_room_gives_nothing() {
    // Arrange
    let mut slots = column(&[300, 300, 300], 8);

    // Act -- window 2 is at its minimum, so window 3 gives it all.
    fit_column(&mut slots, &tall(&[500, 300, 0]), 8);

    // Assert
    assert_eq!(heights(&slots), vec![500, 300, 100]);
}

#[test]
fn the_rounding_remainder_comes_off_a_window_with_room() {
    // Arrange
    let mut slots = column(&[300, 301, 300], 8);

    // Act
    fit_column(&mut slots, &tall(&[400, 0, 0]), 8);

    // Assert -- the height of the column is kept.
    assert_eq!(heights(&slots).iter().sum::<i32>(), 901);
    assert_eq!(slots[0].height, 400);
}

#[test]
fn a_column_that_cannot_hold_its_minimums_is_left_as_it_is() {
    // Arrange
    let mut slots = column(&[300, 300], 8);
    let before = slots.clone();

    // Act
    fit_column(&mut slots, &tall(&[500, 500]), 8);

    // Assert
    assert_eq!(slots, before);
}

#[test]
fn windows_stay_one_gap_apart() {
    // Arrange
    let mut slots = column(&[300, 300, 300], 8);

    // Act
    fit_column(&mut slots, &tall(&[0, 0, 500]), 8);

    // Assert
    for pair in slots.windows(2) {
        assert_eq!(pair[0].y + pair[0].height + 8, pair[1].y);
    }
}
