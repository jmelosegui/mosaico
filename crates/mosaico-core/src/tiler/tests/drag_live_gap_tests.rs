//! A window dragged over another monitor with the pointer in the gap
//! between two tiles there: it keeps the target it had, for the tiles, the
//! preview and the drop, so the tiles do not bounce as the pointer crosses
//! from one tile to the next, as the owner chose (#160 review, round 1,
//! finding 4). Over an outer gap, as over the bar, it joins the end.

use crate::tiler::drag::live::support::{BOTTOM_RIGHT, LEFT, TOP_RIGHT, on_two};
use crate::tiler::drag::support::{center, drop_at, grab, report_at, swapping};
use crate::tiler::tests::MON_W;
use crate::tiler::{Effect, Timing};

/// Between windows 3 and 4 on the second monitor.
const GAP: (i32, i32) = (MON_W + 960, 540);

#[test]
fn in_the_gap_between_two_tiles_the_tiles_stay_as_they_were_laid_out() {
    // Arrange -- over window 3.
    let mut h = on_two(swapping(), &[3, 4]);
    grab(&mut h, 1);
    report_at(&mut h, 1, center(LEFT));

    // Act
    let effects = report_at(&mut h, 1, GAP);

    // Assert -- nothing moves, and the preview stays.
    assert_eq!(h.placements(), vec![]);
    let preview = |e: &Effect| matches!(e, Effect::ShowPreview { .. } | Effect::HidePreview);
    assert!(!effects.iter().any(preview), "{effects:?}");
    assert_eq!((h.rect_of(3), h.rect_of(4)), (TOP_RIGHT, BOTTOM_RIGHT));
}

#[test]
fn a_drop_in_the_gap_between_two_tiles_lands_where_it_was_going() {
    // Arrange -- over window 3, then the gap.
    let mut h = on_two(swapping(), &[3, 4]);
    grab(&mut h, 1);
    report_at(&mut h, 1, center(LEFT));
    report_at(&mut h, 1, GAP);

    // Act
    drop_at(&mut h, 1, GAP);

    // Assert -- before window 3, not at the end.
    assert_eq!(h.active(1), vec![1, 3, 4]);
}

#[test]
fn over_an_outer_gap_the_window_still_joins_the_end() {
    // Arrange -- over window 3.
    let mut h = on_two(swapping(), &[3, 4]);
    grab(&mut h, 1);
    report_at(&mut h, 1, center(LEFT));

    // Act -- between the left edge of the monitor and window 3.
    report_at(&mut h, 1, (MON_W + 4, 540));

    // Assert -- window 3 takes the left half back, window 4 the top of the
    // right, under which window 1 would go.
    let placed = vec![(3, LEFT, Timing::Now), (4, TOP_RIGHT, Timing::Now)];
    assert_eq!(h.placements(), placed);
}
