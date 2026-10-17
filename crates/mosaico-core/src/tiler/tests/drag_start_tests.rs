//! Where and how a drag the tiler follows starts: as the platform saw it
//! start, or as the desktop has the window when it could not tell.

use crate::WindowEvent;
use crate::event::{DragStart, Grab};
use crate::id::wid;
use crate::tiler::tests::Harness;

#[test]
fn a_drag_starts_where_the_platform_saw_the_window_then() {
    // Arrange -- by the time the start is handled, the window has moved on.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let seen = h.rect_of(1);
    h.window(1).rect.x += 30;
    let start = DragStart {
        rect: Some(seen),
        ..DragStart::default()
    };

    // Act
    h.event(WindowEvent::MoveStarted {
        window: wid(1),
        start,
    });

    // Assert
    let drag = h.tiler.mouse.drag.clone().expect("a drag followed");
    assert_eq!(drag.start, seen);
}

#[test]
fn a_drag_that_starts_on_a_maximized_window_moves_it() {
    // Arrange -- whatever the platform says was grabbed, and with nothing
    // read but the pointer.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.window(1).maximized = true;
    let start = DragStart {
        pointer: Some((500, 10)),
        ..DragStart::default()
    };

    // Act
    h.event(WindowEvent::MoveStarted {
        window: wid(1),
        start,
    });

    // Assert -- where it was then, and that it was maximized, come from
    // the desktop.
    let drag = h.tiler.mouse.drag.clone().expect("a drag followed");
    assert_eq!(drag.grab, Some(Grab::TitleBar));
    assert!(drag.from_maximized);
    assert_eq!(drag.start, h.rect_of(1));
}

#[test]
fn a_window_maximized_as_its_drag_started_was_so_once_the_drag_restored_it() {
    // Arrange -- by the time the start is handled, the drag has taken it
    // out of maximized.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let start = DragStart {
        maximized: Some(true),
        ..DragStart::default()
    };

    // Act
    h.event(WindowEvent::MoveStarted {
        window: wid(1),
        start,
    });

    // Assert -- as the platform saw it then.
    let drag = h.tiler.mouse.drag.clone().expect("a drag followed");
    assert_eq!(drag.grab, Some(Grab::TitleBar));
    assert!(drag.from_maximized);
}
