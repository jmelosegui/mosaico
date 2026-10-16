//! The report of a drag the user starts, and the event it becomes.

use mosaico_core::{WindowEvent, WindowId};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_SYSTEM_MOVESIZESTART, OBJID_CARET, OBJID_CLIENT, OBJID_VSCROLL, OBJID_WINDOW,
};

use super::translate;

/// The handle of a top-level window, as the hooks report it.
const TOP: usize = 0x001A_0B2C;

/// What the start of a drag on `object` of the window becomes.
fn drag_started(object: i32) -> Option<WindowEvent> {
    translate(EVENT_SYSTEM_MOVESIZESTART, HWND(TOP as *mut _), object)
}

#[test]
fn a_drag_that_starts_on_a_window_is_reported_for_it() {
    // Act
    let event = drag_started(OBJID_WINDOW.0);

    // Assert
    let window = WindowId::from_raw(TOP as u64);
    assert_eq!(event, Some(WindowEvent::MoveStarted { window }));
}

#[test]
fn a_drag_reported_on_an_inner_object_is_dropped() {
    // Arrange -- only the window itself matters, as for its end.
    for object in [OBJID_CLIENT.0, OBJID_VSCROLL.0, OBJID_CARET.0] {
        // Act
        let event = drag_started(object);

        // Assert
        assert_eq!(event, None, "on {object}");
    }
}
