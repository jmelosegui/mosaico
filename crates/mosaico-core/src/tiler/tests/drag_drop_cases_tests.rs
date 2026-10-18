//! Drops that put the window back in its slot with `drag_to_swap` on: a
//! cancelled drag, one the system snapped and one that resized the
//! window.

use crate::event::{DragStart, Edges, Grab};
use crate::id::wid;
use crate::tiler::tests::{Harness, MON_H, MON_W};
use crate::{Rect, WindowEvent};

use super::super::support::{drop_at, grab, move_to, swapping};

/// Windows 1 and 2 side by side with no gap, so that the pointer at an
/// edge of the monitor is over a tile.
fn edge_to_edge() -> Harness {
    let mut settings = swapping();
    settings.layout.gap = 0;
    Harness::with_settings(settings, 1, &[(1, 0), (2, 0)])
}

#[test]
fn escape_with_the_pointer_over_another_tile_changes_nothing() {
    // Arrange -- dragged over window 2.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let (left, right) = (h.rect_of(1), h.rect_of(2));
    let over = (right.center_x(), right.center_y());
    grab(&mut h, 1);
    move_to(&mut h, 1, over);

    // Act -- the system puts it back where it started.
    h.window(1).rect = left;
    h.window(1).monitor = 0;
    drop_at(&mut h, 1, over);

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!((h.rect_of(1), h.rect_of(2)), (left, right));
}

#[test]
fn a_drop_the_system_snaps_to_a_corner_goes_back_to_its_slot() {
    // Arrange -- dragged to the top right corner, over window 2.
    let mut h = edge_to_edge();
    let left = h.rect_of(1);
    let corner = (MON_W - 1, 0);
    grab(&mut h, 1);
    move_to(&mut h, 1, corner);

    // Act -- the system snaps it to the top right quarter.
    h.window(1).rect = Rect::new(MON_W / 2, 0, MON_W / 2, MON_H / 2);
    drop_at(&mut h, 1, corner);

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!(h.rect_of(1), left);
}

#[test]
fn a_drop_the_system_maximizes_goes_back_to_its_slot() {
    // Arrange -- dragged to the top edge, over window 2.
    let mut h = edge_to_edge();
    let left = h.rect_of(1);
    let top = (MON_W * 3 / 4, 0);
    grab(&mut h, 1);
    move_to(&mut h, 1, top);

    // Act -- the system maximizes it.
    h.window(1).rect = Rect::new(0, 0, MON_W, MON_H);
    h.window(1).maximized = true;
    drop_at(&mut h, 1, top);

    // Assert -- no longer maximized.
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!(h.rect_of(1), left);
    assert!(!h.window(1).maximized);
}

#[test]
fn a_window_resized_by_an_edge_goes_back_to_its_slot() {
    // Arrange -- its right edge dragged far over window 2.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let (left, right) = (h.rect_of(1), h.rect_of(2));
    let edge = Grab::Edges(Edges {
        right: true,
        ..Edges::default()
    });
    let pointer = Some((left.x + left.width, left.center_y()));
    let start = DragStart {
        grab: Some(edge),
        rect: Some(left),
        maximized: Some(false),
        pointer,
    };
    h.event(WindowEvent::MoveStarted {
        window: wid(1),
        start,
    });
    h.window(1).rect.width += 600;

    // Act
    drop_at(&mut h, 1, (right.center_x(), left.center_y()));

    // Assert
    assert_eq!(h.active(0), vec![1, 2]);
    assert_eq!(h.rect_of(1), left);
}
