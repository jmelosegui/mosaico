//! A tiled window dragged over another monitor whose workspace is in
//! monocle, as the owner chose while testing: the workspace makes room as
//! if it were tiled, one window or more, while it stays in monocle, and
//! goes back to monocle as the drag leaves it, for Escape, a key and a
//! click on its bar too, with the monocle window in front and the others
//! where monocle had them.

use crate::action::Origin;
use crate::id::wid;
use crate::tiler::Timing;
use crate::tiler::drag::live::support::{BOTTOM_RIGHT, FRONT, LEFT, RIGHT, monocle_on_second};
use crate::tiler::drag::support::{center, grab, report_at, swapping};
use crate::{Action, Direction, WindowEvent};

#[test]
fn a_monocle_workspace_makes_room_as_if_it_were_tiled() {
    // Arrange -- window 3 in monocle, window 4 behind it where it was
    // tiled.
    let mut h = monocle_on_second(swapping(), &[3, 4]);
    assert_eq!((h.rect_of(3), h.rect_of(4)), (FRONT, RIGHT));
    grab(&mut h, 1);

    // Act -- over where window 4 is tiled.
    report_at(&mut h, 1, center(RIGHT));

    // Assert -- window 1 would go before window 4: window 3 takes the left
    // half and window 4 the bottom of the right one, at once, while the
    // workspace stays in monocle, on window 3.
    let placed = vec![(3, LEFT, Timing::Now), (4, BOTTOM_RIGHT, Timing::Now)];
    assert_eq!(h.placements(), placed);
    let ws = h.workspace(1, 0);
    assert!(ws.monocle());
    assert_eq!(ws.monocle_window(), Some(wid(3)));
    assert_eq!(h.active(1), vec![3, 4]);
}

#[test]
fn leaving_the_monitor_puts_monocle_back_with_its_window_in_front() {
    // Arrange
    let mut h = monocle_on_second(swapping(), &[3, 4]);
    let home = center(h.rect_of(1));
    grab(&mut h, 1);
    report_at(&mut h, 1, center(RIGHT));

    // Act
    report_at(&mut h, 1, home);

    // Assert -- window 3 fills the monitor again, window 4 is back where
    // monocle had it, and focus stays with the window dragged.
    let placed = vec![(3, FRONT, Timing::Now), (4, RIGHT, Timing::Now)];
    assert_eq!(h.placements(), placed);
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
}

#[test]
fn escape_puts_monocle_back() {
    // Arrange
    let mut h = monocle_on_second(swapping(), &[3, 4]);
    let start = h.rect_of(1);
    grab(&mut h, 1);
    report_at(&mut h, 1, center(RIGHT));

    // Act -- the system puts window 1 back where it started.
    h.window(1).rect = start;
    h.window(1).monitor = 0;
    h.event(WindowEvent::LocationChanged { window: wid(1) });

    // Assert
    let placed = vec![(3, FRONT, Timing::Now), (4, RIGHT, Timing::Now)];
    assert_eq!(h.placements(), placed);
}

#[test]
fn a_key_puts_monocle_back() {
    // Arrange
    let mut h = monocle_on_second(swapping(), &[3, 4]);
    grab(&mut h, 1);
    report_at(&mut h, 1, center(RIGHT));

    // Act
    h.action(Action::Focus(Direction::Left));

    // Assert
    let placed = vec![(3, FRONT, Timing::Now), (4, RIGHT, Timing::Now)];
    assert_eq!(h.placements(), placed);
    assert!(h.workspace(1, 0).monocle());
}

#[test]
fn a_click_on_its_bar_that_lays_it_out_puts_monocle_back() {
    // Arrange
    let mut h = monocle_on_second(swapping(), &[3, 4]);
    grab(&mut h, 1);
    report_at(&mut h, 1, center(RIGHT));

    // Act -- on the layout widget of its bar.
    h.action_from(Action::CycleLayout, Origin::Bar(1));

    // Assert -- window 4 goes back where monocle had it, and window 3 is
    // placed once, filling the monitor.
    let placed = h.placements();
    assert!(placed.contains(&(4, RIGHT, Timing::Now)), "{placed:?}");
    let third: Vec<_> = placed.iter().filter(|p| p.0 == 3).collect();
    assert_eq!(third, vec![&(3, FRONT, Timing::Now)]);
}

#[test]
fn a_monocle_workspace_of_one_window_makes_room_and_takes_it_back() {
    // Arrange
    let mut h = monocle_on_second(swapping(), &[3]);
    let home = center(h.rect_of(1));
    grab(&mut h, 1);

    // Act -- over window 3, and back.
    report_at(&mut h, 1, center(FRONT));
    let made_room = h.placements();
    report_at(&mut h, 1, home);

    // Assert -- window 1 would go before it, which takes the right half.
    assert_eq!(made_room, vec![(3, RIGHT, Timing::Now)]);
    assert_eq!(h.placements(), vec![(3, FRONT, Timing::Now)]);
}
