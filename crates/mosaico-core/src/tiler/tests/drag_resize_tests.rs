//! Resizing tiled windows by their edges in BSP, with `resize_splits` on:
//! the edge between two tiles moves the split between them, a corner where
//! two splits meet moves both, and the other edges go back.

use crate::layout::{MAX_SPLIT_RATIO, SplitRatios};
use crate::tiler::tests::Harness;
use crate::{LayoutKind, Rect};

use super::super::support::places;
use super::support::{LEFT, RIGHT, held, resize_to, resizing};

/// Windows `ids` tiled on one monitor in BSP, with splits moved by the
/// edges of the windows the user resizes.
fn bsp(ids: &[u64]) -> Harness {
    let windows: Vec<(u64, usize)> = ids.iter().map(|&id| (id, 0)).collect();
    Harness::with_settings(resizing(), 1, &windows)
}

/// Where the splits of the workspace on screen of `h` are.
fn splits(h: &Harness) -> &SplitRatios {
    h.tiler.monitors()[0].active().split_ratios()
}

#[test]
fn the_edge_between_two_tiles_moves_the_split_between_them() {
    // Arrange
    let mut h = bsp(&[1, 2]);
    let left = h.rect_of(1);

    // Act -- 300 pixels to the right.
    let wider = Rect {
        width: left.width + 300,
        ..left
    };
    let effects = resize_to(&mut h, 1, RIGHT, wider);

    // Assert -- window 2 follows the edge, and both are placed.
    assert_eq!(h.rect_of(1), wider);
    assert_eq!(h.rect_of(2), Rect::new(1264, 8, 648, 1064));
    assert!(places(&effects, 1) && places(&effects, 2), "{effects:?}");
    assert_eq!(h.active(0), vec![1, 2]);
}

#[test]
fn the_window_beyond_a_split_moves_it_with_its_near_edge() {
    // Arrange
    let mut h = bsp(&[1, 2]);
    let right = h.rect_of(2);

    // Act -- its left edge 300 pixels to the left.
    let wider = Rect::new(right.x - 300, right.y, right.width + 300, right.height);
    resize_to(&mut h, 2, LEFT, wider);

    // Assert
    assert_eq!(h.rect_of(2), wider);
    assert_eq!(h.rect_of(1), Rect::new(8, 8, 648, 1064));
}

#[test]
fn a_corner_where_two_splits_meet_moves_both() {
    // Arrange -- window 3 has the bottom of the right half.
    let mut h = bsp(&[1, 2, 3]);
    let corner = held(true, true, false, false);

    // Act -- its top left corner 200 pixels left and 100 up.
    let bigger = Rect::new(764, 444, 1148, 628);
    resize_to(&mut h, 3, corner, bigger);

    // Assert -- window 1 gives up the width, and window 2 the height.
    assert_eq!(h.rect_of(3), bigger);
    assert_eq!(h.rect_of(1), Rect::new(8, 8, 748, 1064));
    assert_eq!(h.rect_of(2), Rect::new(764, 8, 1148, 428));
}

#[test]
fn an_edge_that_borders_no_split_goes_back() {
    // Arrange -- the bottom right corner of window 1, whose bottom edge is
    // an outer one.
    let mut h = bsp(&[1, 2]);
    let left = h.rect_of(1);
    let corner = held(false, false, true, true);

    // Act -- 300 pixels right, and 200 up.
    let dropped = Rect::new(left.x, left.y, left.width + 300, left.height - 200);
    resize_to(&mut h, 1, corner, dropped);

    // Assert -- the split follows the right edge, and the bottom one goes
    // back.
    let expected = Rect {
        width: dropped.width,
        ..left
    };
    assert_eq!(h.rect_of(1), expected);
}

#[test]
fn a_window_resized_by_its_outer_edges_alone_goes_back_and_moves_no_split() {
    // Arrange
    let mut h = bsp(&[1, 2]);
    let left = h.rect_of(1);

    // Act -- its left edge, at the edge of the monitor, 200 pixels right.
    let narrower = Rect::new(left.x + 200, left.y, left.width - 200, left.height);
    let effects = resize_to(&mut h, 1, LEFT, narrower);

    // Assert
    assert_eq!(h.rect_of(1), left);
    assert!(places(&effects, 1), "{effects:?}");
    assert_eq!(splits(&h).ratio(LayoutKind::Bsp, 0), None);
}

#[test]
fn an_edge_dropped_past_the_end_of_its_split_stops_at_nine_tenths() {
    // Arrange
    let mut h = bsp(&[1, 2]);
    let left = h.rect_of(1);

    // Act -- far onto where window 2 is, and past the monitor.
    let wide = Rect {
        width: 3000,
        ..left
    };
    resize_to(&mut h, 1, RIGHT, wide);

    // Assert -- window 2 keeps a tenth.
    assert_eq!(splits(&h).ratio(LayoutKind::Bsp, 0), Some(MAX_SPLIT_RATIO));
    assert_eq!(h.rect_of(1).width, 1709);
    assert_eq!(h.rect_of(2).width, 187);
}
