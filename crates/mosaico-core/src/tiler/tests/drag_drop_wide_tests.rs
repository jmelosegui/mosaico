//! Drops of a window wider than its slot, whose title bar reaches over
//! the tile next to it: a click whose press wobbles, and Esc after a start
//! read a little late, leave it where it was, though it is not in its slot.

use crate::event::{DragStart, Grab};
use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::{Rect, WindowEvent};

use super::super::super::support::{drop_at, move_to, swapping, title_bar};

/// Window 1, focused, 300 pixels wider than its slot, over window 2's.
/// Returns where window 1 is, then window 2's slot.
fn wide() -> (Harness, Rect, Rect) {
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    h.window(1).rect.width += 300;
    let (wide, right) = (h.rect_of(1), h.rect_of(2));
    (h, wide, right)
}

/// The user grabs window 1 by its title bar at `at`, as the platform saw
/// the window at `rect`.
fn grab_at(h: &mut Harness, rect: Rect, at: (i32, i32)) {
    let start = DragStart {
        grab: Some(Grab::TitleBar),
        rect: Some(rect),
        maximized: Some(false),
        pointer: Some(at),
    };
    let window = wid(1);
    h.event(WindowEvent::MoveStarted { window, start });
}

#[test]
fn a_click_whose_press_wobbles_over_the_next_tile_changes_nothing() {
    // Arrange -- pressed on its title bar over window 2's slot.
    let (mut h, wide, right) = wide();
    let at = (right.x + 100, wide.y + 10);
    grab_at(&mut h, wide, at);

    // Act -- the window and the pointer move a pixel before the release.
    h.window(1).rect.x += 1;
    drop_at(&mut h, 1, (at.0 + 1, at.1));

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
}

#[test]
fn escape_after_a_start_read_a_few_pixels_late_changes_nothing() {
    // Arrange -- the start was read once the window had moved 4 pixels,
    // and it is dragged over window 2.
    let (mut h, wide, right) = wide();
    let mut late = wide;
    late.x += 4;
    grab_at(&mut h, late, title_bar(late));
    move_to(&mut h, 1, (right.center_x(), right.center_y()));

    // Act -- Esc: the system puts it back where it was.
    h.window(1).rect = wide;
    h.window(1).monitor = 0;
    drop_at(&mut h, 1, (right.center_x(), right.center_y()));

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
}

#[test]
fn escape_after_a_start_handled_once_the_window_moved_on_changes_nothing() {
    // Arrange -- the platform saw the start where the window was, and the
    // tiler handles it once the window has moved 30 pixels.
    let (mut h, wide, right) = wide();
    h.window(1).rect.x += 30;
    grab_at(&mut h, wide, title_bar(wide));
    move_to(&mut h, 1, (right.center_x(), right.center_y()));

    // Act -- Esc: the system puts it back where it was.
    h.window(1).rect = wide;
    h.window(1).monitor = 0;
    drop_at(&mut h, 1, (right.center_x(), right.center_y()));

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
}
