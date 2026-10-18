//! The drags that show no preview: resizes, and a pointer off every
//! monitor.

use crate::WindowEvent;
use crate::event::{DragStart, Edges, Grab};
use crate::id::wid;
use crate::tiler::Effect;
use crate::tiler::tests::{Harness, MON_H};

use super::super::support::{grab, move_to, swapping};

/// The user starts dragging window `id` of `h` by `grab`, from where it
/// is.
fn start(h: &mut Harness, id: u64, grab: Option<Grab>) {
    let rect = h.rect_of(id);
    let pointer = Some((rect.center_x(), rect.y + 10));
    let start = DragStart {
        grab,
        rect: Some(rect),
        maximized: Some(false),
        pointer,
    };
    let window = wid(id);
    h.event(WindowEvent::MoveStarted { window, start });
}

/// Window `id` reports where it is, with the pointer at `at`. Returns
/// whether a preview shows then.
fn reported(h: &mut Harness, id: u64, at: (i32, i32)) -> bool {
    h.desktop.cursor = Some(at);
    h.event(WindowEvent::LocationChanged { window: wid(id) });
    h.desktop.preview.is_some()
}

#[test]
fn a_window_resized_from_an_edge_shows_no_preview() {
    // Arrange -- its right edge dragged over window 2.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let right = h.rect_of(2);
    let edge = Grab::Edges(Edges {
        right: true,
        ..Edges::default()
    });
    start(&mut h, 1, Some(edge));
    h.window(1).rect.width += 600;

    // Act
    let shows = reported(&mut h, 1, (right.center_x(), right.center_y()));

    // Assert
    assert!(!shows);
}

#[test]
fn without_a_grab_the_preview_shows_until_the_window_changes_size() {
    // Arrange -- the platform could not tell what was grabbed.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let right = h.rect_of(2);
    let over = (right.center_x(), right.center_y());
    start(&mut h, 1, None);

    // Act -- moved, then resized.
    move_to(&mut h, 1, over);
    let while_moved = reported(&mut h, 1, over);
    h.window(1).rect.width += 40;
    let once_resized = reported(&mut h, 1, over);

    // Assert
    assert!(while_moved);
    assert!(!once_resized);
}

#[test]
fn a_pointer_off_every_monitor_hides_the_preview() {
    // Arrange -- shown over window 2.
    let mut h = Harness::with_settings(swapping(), 1, &[(1, 0), (2, 0)]);
    let right = h.rect_of(2);
    grab(&mut h, 1);
    let shown = reported(&mut h, 1, (right.center_x(), right.center_y()));

    // Act -- below the monitor, where a taskbar is.
    h.desktop.cursor = Some((right.center_x(), MON_H + 20));
    let effects = h
        .event(WindowEvent::LocationChanged { window: wid(1) })
        .to_vec();

    // Assert
    assert!(shown);
    assert_eq!(effects, [Effect::HidePreview]);
}
