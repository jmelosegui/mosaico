//! What the start and the end of a drag carry once the hook has read
//! them, on a window of the test's own that is never shown, and the
//! events it leaves alone.

use crate::window::test_windows::HiddenWindow;

use super::*;

#[test]
fn the_start_of_a_drag_carries_where_the_window_is() {
    // Arrange -- a window of our own, never shown, whose client area is
    // all of it. Without a desktop to make it on, skip.
    let Some(window) = HiddenWindow::new() else {
        return;
    };
    let reported = WindowEvent::MoveStarted {
        window: window.id(),
        start: DragStart::default(),
    };

    // Act
    let read = read(reported, window.hwnd());

    // Assert -- its bounds as the desktop reads them, that it is not
    // maximized, and nothing grabbed in a client area.
    let WindowEvent::MoveStarted { window: id, start } = read else {
        panic!("the start of a drag became {read:?}");
    };
    assert_eq!(id, window.id());
    let bounds = Window::new(window.hwnd()).rect().ok();
    assert!(bounds.is_some());
    assert_eq!(start.rect, bounds);
    assert_eq!(start.maximized, Some(false));
    assert_eq!(start.grab, None);
}

#[test]
fn the_end_of_a_drag_carries_where_the_pointer_is() {
    // Arrange -- skip where the pointer cannot be read, as on a desktop
    // that does not take input.
    let Some(window) = HiddenWindow::new() else {
        return;
    };
    if crate::pointer::position().is_err() {
        return;
    }
    let reported = WindowEvent::Moved {
        window: window.id(),
        pointer: None,
    };

    // Act
    let read = read(reported, window.hwnd());

    // Assert -- wherever the pointer is now.
    let WindowEvent::Moved {
        window: id,
        pointer,
    } = read
    else {
        panic!("the end of a drag became {read:?}");
    };
    assert_eq!(id, window.id());
    assert!(pointer.is_some());
}

#[test]
fn events_other_than_a_drag_are_left_as_they_are() {
    // Arrange -- the near miss: a window that is not dragged.
    let Some(window) = HiddenWindow::new() else {
        return;
    };
    let focused = WindowEvent::Focused {
        window: window.id(),
    };

    // Act
    let read = read(focused.clone(), window.hwnd());

    // Assert
    assert_eq!(read, focused);
}
