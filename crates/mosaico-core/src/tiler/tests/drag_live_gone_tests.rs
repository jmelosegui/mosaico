//! How the tiles of another monitor that made room for a dragged window go
//! back to their slots when the drag ends without a drop: the window
//! closes or is minimized, or another window takes focus.

use crate::id::wid;
use crate::tiler::drag::live::support::{BOTTOM_RIGHT, LEFT, RIGHT, TOP_RIGHT, on_two};
use crate::tiler::drag::support::{center, grab, report_at, swapping};
use crate::tiler::tests::Harness;
use crate::{Rect, WindowEvent};

/// Windows 3 and 4 on the second monitor, made room for window 1 dragged
/// over window 3.
fn made_room() -> Harness {
    let mut h = on_two(swapping(), &[3, 4]);
    grab(&mut h, 1);
    report_at(&mut h, 1, center(LEFT));
    assert_eq!((h.rect_of(3), h.rect_of(4)), (TOP_RIGHT, BOTTOM_RIGHT));
    h
}

/// Where windows 3 and 4 are.
fn tiles(h: &mut Harness) -> (Rect, Rect) {
    (h.rect_of(3), h.rect_of(4))
}

#[test]
fn the_dragged_window_closing_puts_the_tiles_back() {
    // Arrange
    let mut h = made_room();

    // Act
    h.event(WindowEvent::Destroyed { window: wid(1) });

    // Assert
    assert_eq!(tiles(&mut h), (LEFT, RIGHT));
}

#[test]
fn the_dragged_window_minimized_puts_the_tiles_back() {
    // Arrange
    let mut h = made_room();

    // Act
    h.window(1).minimized = true;
    h.event(WindowEvent::Minimized { window: wid(1) });

    // Assert
    assert_eq!(tiles(&mut h), (LEFT, RIGHT));
}

#[test]
fn another_window_taking_focus_puts_the_tiles_back() {
    // Arrange
    let mut h = made_room();

    // Act -- the user clicks window 2.
    h.event(WindowEvent::Focused { window: wid(2) });

    // Assert
    assert_eq!(tiles(&mut h), (LEFT, RIGHT));
}
