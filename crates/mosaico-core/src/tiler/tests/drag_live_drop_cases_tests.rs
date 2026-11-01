//! The tiles of another monitor while a window is dragged over them, in
//! the cases left as before, with `drag_to_swap` off and for windows the
//! tiler does not follow the drag of; for many reports over the same
//! tile; and with a window there minimized without a report.

use crate::WindowEvent;
use crate::id::wid;
use crate::tiler::drag::live::support::{RIGHT, on_two, place_any};
use crate::tiler::drag::support::{center, drop_at, grab, report_at, swapping};
use crate::tiler::tests::fake_desktop::FakeWindow;
use crate::tiler::{Effect, Settings, Timing};

#[test]
fn with_drag_to_swap_off_the_tiles_of_another_monitor_stay() {
    // Arrange
    let mut h = on_two(Settings::with_gap(8), &[3, 4]);
    let over = center(h.rect_of(3));
    grab(&mut h, 1);

    // Act
    let effects = report_at(&mut h, 1, over);

    // Assert
    assert!(!place_any(&effects), "{effects:?}");
}

#[test]
fn a_window_the_tiler_leaves_alone_moves_no_tile_as_it_is_dragged_over_them() {
    // Arrange -- a window no rule manages.
    let mut h = on_two(swapping(), &[3, 4]);
    let mut floating = FakeWindow::app(0);
    floating.facts = None;
    h.desktop.windows.insert(wid(9), floating);
    h.event(WindowEvent::move_started(wid(9)));

    // Act -- over window 3.
    let over = center(h.rect_of(3));
    let effects = report_at(&mut h, 9, over);

    // Assert
    assert!(!place_any(&effects), "{effects:?}");
}

#[test]
fn a_window_the_tiler_would_tile_but_does_not_manage_yet_moves_no_tile_either() {
    // Arrange -- never reported.
    let mut h = on_two(swapping(), &[3, 4]);
    h.desktop.windows.insert(wid(9), FakeWindow::app(0));
    h.event(WindowEvent::move_started(wid(9)));

    // Act
    let over = center(h.rect_of(3));
    let effects = report_at(&mut h, 9, over);

    // Assert
    assert!(!place_any(&effects), "{effects:?}");
}

#[test]
fn many_reports_over_the_same_tile_place_nothing_after_the_first() {
    // Arrange
    let mut h = on_two(swapping(), &[3, 4]);
    let (x, y) = center(h.rect_of(3));
    grab(&mut h, 1);
    report_at(&mut h, 1, (x, y));
    let first = h.placements();

    // Act -- on over window 3, a few pixels at a time.
    let later: Vec<Vec<Effect>> = (1..=5)
        .map(|step| report_at(&mut h, 1, (x + 7 * step, y - 3 * step)))
        .collect();

    // Assert
    assert_eq!(first.len(), 2, "{first:?}");
    assert!(later.iter().all(|effects| !place_any(effects)), "{later:?}");
}

#[test]
fn a_window_there_minimized_without_a_report_leaves_before_the_tiles_make_room() {
    // Arrange
    let mut h = on_two(swapping(), &[3, 4]);
    let over = center(h.rect_of(3));
    h.window(4).minimized = true;
    grab(&mut h, 1);

    // Act
    report_at(&mut h, 1, over);

    // Assert -- window 3 makes room for window 1 alone, and window 4 is
    // left minimized.
    assert_eq!(h.placements(), vec![(3, RIGHT, Timing::Now)]);
    assert_eq!(h.active(1), vec![3]);
    assert!(h.window(4).minimized);
}

#[test]
fn over_the_slot_of_a_window_there_minimized_without_a_report_the_tiles_make_room_as_the_drop() {
    // Arrange -- window 4 minimized without a report, the pointer over
    // where it was laid out.
    let mut h = on_two(swapping(), &[3, 4]);
    let over = center(RIGHT);
    h.window(4).minimized = true;
    grab(&mut h, 1);

    // Act
    report_at(&mut h, 1, over);

    // Assert -- window 3, now alone there, keeps the left half, as window
    // 1 would join after it, under the pointer, as a drop there would
    // (#160 review, round 1, finding 3).
    assert_eq!(h.placements(), vec![]);
    drop_at(&mut h, 1, over);
    assert_eq!(h.active(1), vec![3, 1]);
}
