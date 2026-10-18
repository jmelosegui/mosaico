//! How the tiles of another monitor that made room for a dragged window
//! go back: for Escape and for a key, to their slots wherever they were as
//! the drag began, after a layout of the window's own monitor meanwhile;
//! and how they stay where they made room under a window the system
//! maximizes there as it is dropped.

use crate::id::wid;
use crate::tiler::Timing;
use crate::tiler::drag::live::support::{LEFT, RIGHT, TOP_RIGHT, on_two, place_any};
use crate::tiler::drag::support::{center, drop_at, grab, move_to, report_at, swapping};
use crate::tiler::tests::{MON_H, MON_W};
use crate::{Action, Direction, Rect, WindowEvent};

#[test]
fn escape_puts_the_tiles_of_the_other_monitor_back() {
    // Arrange
    let mut h = on_two(swapping(), &[3, 4]);
    let (start, over) = (h.rect_of(1), center(h.rect_of(3)));
    grab(&mut h, 1);
    report_at(&mut h, 1, over);

    // Act -- the system puts window 1 back where it started, with the
    // pointer still over window 3.
    h.window(1).rect = start;
    h.window(1).monitor = 0;
    h.event(WindowEvent::LocationChanged { window: wid(1) });

    // Assert
    let placed = vec![(3, LEFT, Timing::Now), (4, RIGHT, Timing::Now)];
    assert_eq!(h.placements(), placed);
}

#[test]
fn a_key_puts_the_tiles_of_the_other_monitor_back() {
    // Arrange
    let mut h = on_two(swapping(), &[3, 4]);
    let over = center(h.rect_of(3));
    grab(&mut h, 1);
    report_at(&mut h, 1, over);

    // Act -- a key that lays nothing out of its own, and the drag goes on.
    h.action(Action::Focus(Direction::Left));
    let placed = h.placements();
    let later = report_at(&mut h, 1, (over.0 + 20, over.1));

    // Assert -- and the drag no longer moves them.
    assert_eq!(
        placed,
        vec![(3, LEFT, Timing::Now), (4, RIGHT, Timing::Now)]
    );
    assert!(!place_any(&later), "{later:?}");
}

#[test]
fn the_tiles_go_back_to_their_slots_wherever_they_were_as_the_drag_began() {
    // Arrange -- window 4 off its slot, as on its way there.
    let mut h = on_two(swapping(), &[3, 4]);
    let (home, over) = (center(h.rect_of(1)), center(h.rect_of(3)));
    h.window(4).rect = Rect::new(MON_W + 900, 100, 600, 600);
    grab(&mut h, 1);
    report_at(&mut h, 1, over);

    // Act
    report_at(&mut h, 1, home);

    // Assert
    assert_eq!(h.rect_of(4), RIGHT);
}

#[test]
fn a_layout_of_the_window_s_own_monitor_meanwhile_leaves_the_others_to_go_back() {
    // Arrange -- over window 3, and window 5 opens on the first monitor,
    // which lays it out.
    let mut h = on_two(swapping(), &[3, 4]);
    let (home, over) = (center(h.rect_of(1)), center(h.rect_of(3)));
    grab(&mut h, 1);
    report_at(&mut h, 1, over);
    h.open(5, 0);

    // Act -- back over its own slot.
    report_at(&mut h, 1, home);

    // Assert
    let placed = vec![(3, LEFT, Timing::Now), (4, RIGHT, Timing::Now)];
    assert_eq!(h.placements(), placed);
}

#[test]
fn a_window_the_system_maximizes_as_it_is_dropped_there_leaves_the_tiles_where_they_made_room() {
    // Arrange -- over the top edge of the second monitor, where window 1
    // would join the end, under window 4.
    let mut h = on_two(swapping(), &[3, 4]);
    let top = (MON_W + MON_W / 2, 0);
    grab(&mut h, 1);
    report_at(&mut h, 1, top);
    let made_room = h.rect_of(4);

    // Act -- the system maximizes it there as it is dropped.
    move_to(&mut h, 1, top);
    h.window(1).rect = Rect::new(MON_W, 0, MON_W, MON_H);
    h.window(1).maximized = true;
    drop_at(&mut h, 1, top);

    // Assert -- it stays maximized, and window 4 where it made room.
    assert_eq!(h.active(1), vec![3, 4, 1]);
    assert!(h.window(1).maximized);
    assert_eq!((made_room, h.rect_of(4)), (TOP_RIGHT, TOP_RIGHT));
}
