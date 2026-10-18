//! How the tile that follows a held edge ends: the drop finds it in
//! place, while Escape, a key, a display change noted and another drag in
//! place of one whose end was lost put it back in its slot, monocle turned
//! on puts it back where it was, and the tiler stopping leaves it where it
//! is.

use crate::event::{DragStart, Grab};
use crate::id::wid;
use crate::tiler::drag::live::support::{border_at, following, pair, report, widened};
use crate::tiler::drag::resize::support::{RIGHT, grab_to, holding, resizing, unmoved};
use crate::tiler::tests::Harness;
use crate::tiler::{Effect, Input, Timing};
use crate::{Action, Direction, Rect, WindowEvent};

#[test]
fn each_report_moves_the_tile_on_and_the_drop_leaves_it_where_it_is() {
    // Arrange
    let mut h = pair(resizing());
    grab_to(&mut h, 1, RIGHT, widened(100));

    // Act -- on to 300 pixels wider, and let go there.
    let followed = report(&mut h, widened(300));
    let pointer = Some(holding(widened(300), RIGHT));
    let dropped = h.event(WindowEvent::Moved {
        window: wid(1),
        pointer,
    });
    let at_drop: Vec<Rect> = dropped
        .iter()
        .filter_map(|e| match *e {
            Effect::Place { window, rect, .. } if window == wid(2) => Some(rect),
            _ => None,
        })
        .collect();

    // Assert -- the drop places window 2 where it already is, and moves
    // the split there.
    let beyond = Rect::new(1264, 8, 648, 1064);
    assert_eq!(followed, vec![(2, beyond, Timing::Now)]);
    assert_eq!(at_drop, vec![beyond]);
    assert!(!unmoved(&h));
}

#[test]
fn escape_puts_the_tile_back_in_its_slot() {
    // Arrange
    let mut h = pair(resizing());
    let (left, right) = (h.rect_of(1), h.rect_of(2));
    grab_to(&mut h, 1, RIGHT, widened(200));
    assert_eq!(h.rect_of(2), following());

    // Act -- the system puts window 1 back where it started.
    let placed = report(&mut h, left);

    // Assert -- with its border.
    assert_eq!(placed, vec![(2, right, Timing::Now)]);
    assert_eq!(border_at(&h.effects, 2), Some(right));
    assert!(unmoved(&h));
}

#[test]
fn a_key_puts_the_tile_back_in_its_slot_and_the_drag_moves_it_no_more() {
    // Arrange
    let mut h = pair(resizing());
    let right = h.rect_of(2);
    grab_to(&mut h, 1, RIGHT, widened(200));

    // Act -- a key that lays nothing out of its own, and the drag goes on.
    h.action(Action::Focus(Direction::Left));
    let placed = h.placements();
    let later = report(&mut h, widened(300));

    // Assert
    assert_eq!(placed, vec![(2, right, Timing::Now)]);
    assert!(later.is_empty(), "{later:?}");
}

#[test]
fn a_display_change_noted_mid_drag_puts_the_tile_back() {
    // Arrange
    let mut h = pair(resizing());
    let right = h.rect_of(2);
    grab_to(&mut h, 1, RIGHT, widened(200));

    // Act
    h.step(Input::DisplayChanging);

    // Assert
    assert_eq!(h.placements(), vec![(2, right, Timing::Now)]);
}

#[test]
fn monocle_turned_on_mid_drag_puts_the_tile_back_behind_the_window_held() {
    // Arrange
    let mut h = pair(resizing());
    let right = h.rect_of(2);
    grab_to(&mut h, 1, RIGHT, widened(200));

    // Act -- monocle, for window 1, which the layout leaves to the user.
    h.action(Action::ToggleMonocle);

    // Assert -- window 2 goes back where it was, behind it.
    assert!(h.workspace(0, 0).monocle());
    assert_eq!(h.placements(), vec![(2, right, Timing::Now)]);
}

#[test]
fn the_tiler_stopping_mid_drag_leaves_the_tile_where_it_followed() {
    // Arrange
    let mut h = pair(resizing());
    grab_to(&mut h, 1, RIGHT, widened(200));

    // Act
    h.step(Input::Shutdown);

    // Assert
    assert!(h.placements().is_empty(), "{:?}", h.effects);
    assert_eq!(h.rect_of(2), following());
}

#[test]
fn another_drag_in_place_of_one_whose_end_was_lost_puts_the_tiles_back() {
    // Arrange -- windows 2 and 3 follow the right edge of window 1, and
    // the end of that drag is never reported.
    let mut h = Harness::with_settings(resizing(), 1, &[(1, 0), (2, 0), (3, 0)]);
    let top = h.rect_of(2);
    grab_to(&mut h, 1, RIGHT, widened(200));

    // Act -- the user grabs window 3 by its title bar.
    let start = DragStart {
        grab: Some(Grab::TitleBar),
        rect: Some(h.rect_of(3)),
        maximized: Some(false),
        pointer: None,
    };
    h.event(WindowEvent::MoveStarted {
        window: wid(3),
        start,
    });

    // Assert -- window 2 goes back to its slot, and window 3, which the
    // user has now, is left alone.
    assert_eq!(h.placements(), vec![(2, top, Timing::Now)]);
}
