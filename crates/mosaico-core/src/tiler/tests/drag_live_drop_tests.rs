//! The tiles of another monitor while a tiled window is dragged over
//! them, with `drag_to_swap` on: laid out at once as if the window had
//! joined there, before the tile under the pointer or at the end over the
//! bar, and again as the pointer moves on to another tile there, while
//! nothing is committed until the drop, which finds them in place.

use crate::Rect;
use crate::id::wid;
use crate::tiler::drag::live::support::{BOTTOM_RIGHT, LEFT, RIGHT, TOP_RIGHT, on_two};
use crate::tiler::drag::support::{center, drop_at, grab, report_at, reserve_bar, swapping};
use crate::tiler::tests::MON_W;
use crate::tiler::{Effect, Timing};

#[test]
fn the_tiles_of_another_monitor_make_room_for_a_window_dragged_over_them() {
    // Arrange
    let mut h = on_two(swapping(), &[3, 4]);
    let over = center(h.rect_of(3));
    grab(&mut h, 1);

    // Act
    report_at(&mut h, 1, over);

    // Assert -- windows 3 and 4 take, at once, the slots they get with
    // window 1 before window 3: the right half, shared. Window 1 is left
    // where the user has it, and no workspace has changed.
    let placed = vec![(3, TOP_RIGHT, Timing::Now), (4, BOTTOM_RIGHT, Timing::Now)];
    assert_eq!(h.placements(), placed);
    assert_eq!((h.active(0), h.active(1)), (vec![1, 2], vec![3, 4]));
}

#[test]
fn moving_on_to_another_tile_there_lays_its_tiles_out_again() {
    // Arrange -- over window 3.
    let mut h = on_two(swapping(), &[3, 4]);
    let (third, fourth) = (center(h.rect_of(3)), center(h.rect_of(4)));
    grab(&mut h, 1);
    report_at(&mut h, 1, third);

    // Act -- on to where window 4 is laid out, and back.
    report_at(&mut h, 1, fourth);
    let before_fourth = h.placements();
    report_at(&mut h, 1, third);

    // Assert -- before window 4, window 3 takes the left half back, and
    // window 4 stays; before window 3 again, it takes the top right.
    assert_eq!(before_fourth, vec![(3, LEFT, Timing::Now)]);
    assert_eq!(h.placements(), vec![(3, TOP_RIGHT, Timing::Now)]);
    assert_eq!(h.rect_of(4), BOTTOM_RIGHT);
}

#[test]
fn moving_back_over_its_own_monitor_puts_the_tiles_there_back() {
    // Arrange
    let mut h = on_two(swapping(), &[3, 4]);
    let (home, over) = (center(h.rect_of(1)), center(h.rect_of(3)));
    grab(&mut h, 1);
    report_at(&mut h, 1, over);

    // Act
    report_at(&mut h, 1, home);

    // Assert
    let placed = vec![(3, LEFT, Timing::Now), (4, RIGHT, Timing::Now)];
    assert_eq!(h.placements(), placed);
}

#[test]
fn the_drop_finds_the_tiles_there_in_place() {
    // Arrange
    let mut h = on_two(swapping(), &[3, 4]);
    let over = center(h.rect_of(3));
    grab(&mut h, 1);
    report_at(&mut h, 1, over);

    // Act
    let effects = drop_at(&mut h, 1, over);

    // Assert -- window 1 joins before window 3, and the drop places window
    // 3 where the drag put it, as window 4.
    assert_eq!(h.active(1), vec![1, 3, 4]);
    let at = |id: u64| {
        effects.iter().find_map(|e| match *e {
            Effect::Place { window, rect, .. } if window == wid(id) => Some(rect),
            _ => None,
        })
    };
    assert_eq!((at(3), at(4)), (Some(TOP_RIGHT), Some(BOTTOM_RIGHT)));
    assert_eq!(h.rect_of(1), LEFT);
}

#[test]
fn over_the_bar_of_another_monitor_its_tiles_make_room_at_the_end() {
    // Arrange -- a 30 pixel bar on every monitor.
    let mut h = on_two(swapping(), &[3, 4]);
    reserve_bar(&mut h);
    grab(&mut h, 1);

    // Act -- over the bar of the second monitor.
    report_at(&mut h, 1, (MON_W + 100, 15));

    // Assert -- window 1 would join the end, under window 4, which takes
    // the top of the right half; window 3 keeps the left half.
    let top = Rect::new(MON_W + 964, 38, 948, 513);
    assert_eq!(h.placements(), vec![(4, top, Timing::Now)]);
}
