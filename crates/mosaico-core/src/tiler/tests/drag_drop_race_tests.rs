//! Drops decided from what the platform read before the tiler handled the
//! drag: a start read once the window had moved, a maximize the drag undid
//! before its start was handled, one a double click made before the end
//! of its first click was, and a size a monitor of another scale rounded.

use crate::event::{DragStart, Grab};
use crate::id::wid;
use crate::tiler::tests::{Harness, MON_H, MON_W};
use crate::{Rect, WindowEvent};

use super::super::super::support::{drop_at, grab, move_to, places, swapping, title_bar};

/// Window 1, focused, and window 2 side by side, windows dropped on tiles
/// moving there.
fn two() -> Harness {
    Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)])
}

/// The user grabs window 1 by its title bar, as the platform saw it then.
fn start(h: &mut Harness, rect: Rect, maximized: bool) {
    let start = DragStart {
        grab: Some(Grab::TitleBar),
        rect: Some(rect),
        maximized: Some(maximized),
        pointer: Some(title_bar(rect)),
    };
    let window = wid(1);
    h.event(WindowEvent::MoveStarted { window, start });
}

#[test]
fn a_maximized_window_the_drag_restored_before_its_start_was_handled_still_swaps() {
    // Arrange -- window 1 is maximized as the user grabs it, and restored
    // under the pointer by the time the start is handled.
    let mut h = two();
    let right = h.rect_of(2);
    let all = Rect::new(0, 0, MON_W, MON_H);
    h.window(1).rect = all;
    h.window(1).maximized = true;
    h.event(WindowEvent::LocationChanged { window: wid(1) });
    h.window(1).rect = Rect::new(title_bar(all).0 - 240, 0, 480, 750);
    h.window(1).maximized = false;
    start(&mut h, all, true);

    // Act -- dropped over window 2.
    let over = (right.center_x(), right.center_y());
    move_to(&mut h, 1, over);
    drop_at(&mut h, 1, over);

    // Assert -- as the preview showed.
    assert_eq!(h.active(0), vec![2, 1]);
}

#[test]
fn the_first_click_of_a_double_click_on_a_title_bar_leaves_the_window_maximized() {
    // Arrange -- the platform reports the first click as a drag that moves
    // nothing, and the second click has maximized the window by the time
    // the end of the first is handled.
    let mut h = two();
    let slot = h.rect_of(1);
    start(&mut h, slot, false);
    h.window(1).rect = Rect::new(0, 0, MON_W, MON_H);
    h.window(1).maximized = true;

    // Act
    let effects = drop_at(&mut h, 1, title_bar(slot));

    // Assert -- as before drops moved windows.
    assert!(!places(&effects, 1), "{effects:?}");
    assert!(h.window(1).maximized);
}

#[test]
fn escape_after_a_start_read_late_changes_nothing_with_the_pointer_over_another_tile() {
    // Arrange -- the start was read once the window had moved 4 pixels,
    // and the window is dragged over window 2.
    let mut h = two();
    let (left, right) = (h.rect_of(1), h.rect_of(2));
    let mut late = left;
    late.x += 4;
    start(&mut h, late, false);
    let over = (right.center_x(), right.center_y());
    move_to(&mut h, 1, over);

    // Act -- Esc: the system puts it back in its slot.
    h.window(1).rect = left;
    h.window(1).monitor = 0;
    drop_at(&mut h, 1, over);

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!((h.rect_of(1), h.rect_of(2)), (left, right));
}

#[test]
fn a_window_a_pixel_wider_after_its_move_still_swaps() {
    // Arrange -- dragged over window 2, and a pixel wider, as a trip
    // across a monitor of another scale leaves it.
    let mut h = two();
    let right = h.rect_of(2);
    let over = (right.center_x(), right.center_y());
    grab(&mut h, 1);
    move_to(&mut h, 1, over);
    h.window(1).rect.width += 1;

    // Act
    drop_at(&mut h, 1, over);

    // Assert
    assert_eq!(h.active(0), vec![2, 1]);
}
