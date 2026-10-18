use crate::Rect;
use crate::event::Grab;
use crate::id::wid;

use super::super::Drag;
use super::*;

/// Its slot on monitor 0, where window 1 was as its drag started.
const SLOT: Rect = Rect {
    x: 10,
    y: 40,
    width: 480,
    height: 750,
};

/// Window 1, dragged from its slot by its title bar, with the pointer at
/// 250, 50 as the drag started.
fn drag() -> Drag {
    Drag {
        window: wid(1),
        monitor: 0,
        grab: Some(Grab::TitleBar),
        from_maximized: false,
        start: SLOT,
        pointer: Some((250, 50)),
        focus_owed: false,
        decides: true,
        preview: None,
    }
}

#[test]
fn a_window_left_where_it_started_was_not_moved() {
    // Act -- Esc puts it back, wherever the pointer is, and whether the
    // window has a slot or not.
    let ended = cancelled(&drag(), SLOT, Some((700, 400)), None);

    // Assert
    assert!(ended);
}

#[test]
fn a_window_put_back_in_its_slot_after_a_late_start_was_not_moved() {
    // Arrange -- the start was read once the window had moved 4 pixels.
    let drag = Drag {
        start: Rect::new(14, 40, 480, 750),
        ..drag()
    };

    // Act -- Esc puts it back in its slot.
    let ended = cancelled(&drag, SLOT, Some((700, 400)), Some(SLOT));

    // Assert
    assert!(ended);
}

#[test]
fn a_drag_whose_pointer_ends_where_it_started_was_a_click() {
    // Arrange -- the second click of a double click on the title bar
    // maximized the window before the first one's end is handled.
    let all = Rect::new(0, 30, 1000, 770);

    // Act
    let ended = cancelled(&drag(), all, Some((250, 50)), Some(SLOT));

    // Assert
    assert!(ended);
}

#[test]
fn a_window_moved_elsewhere_was_not_cancelled() {
    // Arrange -- the near misses: moved, and with no pointer read at the
    // start or at the end.
    let moved = Rect::new(300, 140, 480, 750);
    let unread = Drag {
        pointer: None,
        ..drag()
    };

    // Act
    let by_pointer = cancelled(&drag(), moved, Some((540, 150)), Some(SLOT));
    let unread_pointers = cancelled(&unread, moved, None, Some(SLOT));

    // Assert
    assert!(!by_pointer);
    assert!(!unread_pointers);
}

#[test]
fn a_few_pixels_off_where_it_started_is_still_where_it_started() {
    // Arrange -- the window, then the pointer, 4 pixels off, then 5.
    let moved = |by: i32| Rect::new(10 + by, 40, 480, 750);
    let pointer = |by: i32| Some((250 + by, 50 - by));
    let elsewhere = Rect::new(300, 140, 480, 750);

    // Act
    let windows = [4, 5].map(|by| cancelled(&drag(), moved(by), Some((700, 400)), None));
    let pointers = [4, 5].map(|by| cancelled(&drag(), elsewhere, pointer(by), None));

    // Assert
    assert_eq!(windows, [true, false]);
    assert_eq!(pointers, [true, false]);
}
