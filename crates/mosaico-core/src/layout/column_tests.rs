//! A column of windows: shared equally when nobody resized it, and at its
//! heights once someone did, each window ending where its share puts it,
//! to the nearest pixel, a whole gap from the next.

use crate::Rect;

use super::column_slots;

/// A column 1000 pixels tall, from the top of the screen.
const AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 300,
    height: 1000,
};

/// Where `count` windows go down [`AREA`] with `gap`, ending at `ends`, as
/// `(top, height)`.
fn column(count: usize, gap: i32, ends: Option<&[f64]>) -> Vec<(i32, i32)> {
    let slots = column_slots(&AREA, count, gap, ends);
    slots.iter().map(|s| (s.y, s.height)).collect()
}

#[test]
fn a_column_nobody_resized_is_shared_equally_and_the_last_takes_the_rest() {
    // Act -- three windows, 8 apart, which leave 984 pixels to share.
    let slots = column(3, 8, None);

    // Assert
    assert_eq!(slots, [(0, 328), (336, 328), (672, 328)]);
    assert_eq!(column(3, 7, None), [(0, 328), (335, 328), (670, 330)]);
}

#[test]
fn each_window_of_a_resized_column_ends_where_its_share_puts_it() {
    // Act -- four windows, a tenth, four tenths, a third and the rest of
    // the column, no gaps.
    let slots = column(4, 0, Some(&[0.1, 0.5, 0.8333]));

    // Assert -- 833.3 rounds to 833, and the last ends at the bottom.
    assert_eq!(slots, [(0, 100), (100, 400), (500, 333), (833, 167)]);
}

#[test]
fn a_gap_is_kept_whole_on_either_side_of_a_window_of_a_resized_column() {
    // Act -- an odd gap, which leaves 993 pixels to share.
    let slots = column(2, 7, Some(&[0.25]));

    // Assert -- the first ends at 248, and the second starts 7 below it.
    assert_eq!(slots, [(0, 248), (255, 745)]);
}

#[test]
fn an_end_on_half_a_pixel_rounds_down_whatever_the_last_bit_of_its_share() {
    // Arrange -- a column of 1045 pixels, where half of it is 522.5, and
    // the shares a bit under and a bit over a half.
    let area = Rect::new(0, 0, 300, 1045);
    let half: f64 = 0.5;
    let bits = [half.to_bits() - 1, half.to_bits(), half.to_bits() + 1];

    // Act
    let slots = bits.map(|bits| column_slots(&area, 2, 0, Some(&[f64::from_bits(bits)])));

    // Assert -- the first ends at 522 each time.
    let rows = slots.map(|s| s.iter().map(|s| (s.y, s.height)).collect::<Vec<_>>());
    assert!(
        rows.iter().all(|r| r == &[(0, 522), (522, 523)]),
        "{rows:?}"
    );
}

#[test]
fn heights_for_another_number_of_windows_are_left_out() {
    // Act -- too few, and too many.
    let (few, many) = (column(3, 8, Some(&[0.5])), column(2, 8, Some(&[0.3, 0.6])));

    // Assert
    assert_eq!((few, many), (column(3, 8, None), column(2, 8, None)));
}

#[test]
fn a_column_of_no_window_has_no_slot() {
    // Act
    let slots = column(0, 8, Some(&[0.5]));

    // Assert
    assert!(slots.is_empty());
}
