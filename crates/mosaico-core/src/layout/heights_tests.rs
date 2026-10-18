//! The heights of a resized column: shares that sum to 1, each at least a
//! tenth of an equal share, through every change.

use super::{Heights, least};

/// Whether `heights` give every one of `count` windows a share of at least
/// a tenth of an equal one, the shares summing to 1.
fn sound(heights: &Heights, count: usize) -> bool {
    let shares = heights.shares().unwrap_or_default();
    let floor = least(count) - 1e-12;
    let total: f64 = shares.iter().sum();
    shares.len() == count && shares.iter().all(|&s| s >= floor) && (total - 1.0).abs() < 1e-9
}

/// The shares of `heights`, rounded to a millionth to read.
fn shares(heights: &Heights) -> Vec<f64> {
    let round = |share: f64| (share * 1e6).round() / 1e6;
    heights
        .shares()
        .unwrap_or_default()
        .into_iter()
        .map(round)
        .collect()
}

#[test]
fn shares_are_scaled_to_fill_the_column_and_one_window_has_none() {
    assert_eq!(shares(&Heights::of(&[2.0, 1.0, 1.0])), [0.5, 0.25, 0.25]);
    assert_eq!(Heights::of(&[1.0]), Heights::default());
}

#[test]
fn a_share_below_a_tenth_of_an_equal_one_is_raised_and_the_others_give_way() {
    // Act -- three windows, the last with nothing.
    let heights = Heights::of(&[0.6, 0.4, 0.0]);

    // Assert -- it gets a thirtieth, which the others give up in proportion.
    assert_eq!(shares(&heights), [0.58, 0.386667, 0.033333]);
    assert!(sound(&heights, 3));
}

#[test]
fn a_window_that_joins_gets_an_equal_share_and_the_others_shrink_in_proportion() {
    // Arrange
    let mut heights = Heights::of(&[0.5, 0.3, 0.2]);

    // Act -- one joins second.
    heights.join(1);

    // Assert
    assert_eq!(shares(&heights), [0.375, 0.25, 0.225, 0.15]);
}

#[test]
fn a_window_that_leaves_gives_its_share_to_the_others_in_proportion() {
    // Arrange -- the first two of the narrow ones at the floor of three.
    let mut heights = Heights::of(&[0.5, 0.3, 0.2]);
    let mut narrow = Heights::of(&[1.0, 1.0, 28.0]);

    // Act -- the second leaves, and the first of the narrow ones.
    heights.leave(1);
    narrow.leave(0);

    // Assert -- a twenty-ninth would leave the narrow one under the floor
    // of two, a twentieth.
    assert_eq!(shares(&heights), [0.714286, 0.285714]);
    assert_eq!(shares(&narrow), [0.05, 0.95]);
    let mut gone = Heights::of(&[0.7, 0.3]);
    gone.leave(0);
    assert_eq!(gone, Heights::default());
}

#[test]
fn a_moved_end_changes_the_two_windows_beside_it_alone_and_stops_at_the_floor() {
    // Arrange -- four windows, a quarter each.
    let ends = [0.25, 0.5, 0.75];

    // Act -- the second one's end moved down, up past the top of the
    // second, and down past the bottom of the third.
    let down = Heights::moved(&ends, 1, 0.6);
    let up = Heights::moved(&ends, 1, 0.0);
    let past = Heights::moved(&ends, 1, 1.0);

    // Assert -- a fortieth of the column is the least either keeps.
    assert_eq!(shares(&down), [0.25, 0.35, 0.15, 0.25]);
    assert_eq!(shares(&up), [0.25, 0.025, 0.475, 0.25]);
    assert_eq!(shares(&past), [0.25, 0.475, 0.025, 0.25]);
}

#[test]
fn a_whole_column_back_has_its_heights_back_and_part_of_one_shares_the_rest() {
    // Arrange
    let had = Heights::of(&[0.5, 0.3, 0.2]);
    let back: Vec<(usize, f64)> = (0..3).map(|at| (at, had.share(at).unwrap())).collect();

    // Act -- all three back, and the first two of four.
    let mut whole = Heights::default();
    whole.restore(&back, 3);
    let mut part = Heights::default();
    part.restore(&back[..2], 4);

    // Assert -- the two others share the fifth left.
    assert_eq!(shares(&whole), shares(&had));
    assert_eq!(shares(&part), [0.5, 0.3, 0.1, 0.1]);
}

#[test]
fn the_shares_stay_sound_as_windows_join_and_leave() {
    // A small generator of numbers, the same on every run.
    let mut seed = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move |below: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % below.max(1) as u64) as usize
    };
    for _ in 0..300 {
        // Arrange -- shares of any size, some of nothing.
        let mut count = 2 + next(6);
        let shares: Vec<f64> = (0..count).map(|_| next(1000) as f64).collect();
        let mut heights = Heights::of(&shares);
        assert!(sound(&heights, count), "{shares:?}");
        for _ in 0..60 {
            // Act -- a window joins or leaves, anywhere.
            if count < 3 || next(2) == 0 {
                heights.join(next(count + 1));
                count += 1;
            } else {
                heights.leave(next(count));
                count -= 1;
            }

            // Assert
            assert!(sound(&heights, count), "{heights:?} of {count}");
        }
    }
}
