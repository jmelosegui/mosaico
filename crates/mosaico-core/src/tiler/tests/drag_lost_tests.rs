//! A drag whose end is never reported: another drag, or another window
//! taking focus other than by the tiler's doing, ends it.

use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::tiler::tests::fake_desktop::FakeWindow;
use crate::{Action, Direction, WindowEvent};

use super::super::support::{grab, places};

#[test]
fn a_new_drag_takes_the_place_of_one_whose_end_was_never_reported() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    grab(&mut h, 1);
    grab(&mut h, 2);

    // Act
    let effects = h.open(3, 0).to_vec();

    // Assert -- only the window dragged last keeps its slot to itself.
    assert!(places(&effects, 1), "{effects:?}");
    assert!(!places(&effects, 2), "{effects:?}");
}

#[test]
fn another_window_taking_focus_ends_the_drag() {
    // Arrange -- the user drags window 1, which has focus.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    grab(&mut h, 1);

    // Act -- they click window 2, then a window opens.
    h.event(WindowEvent::Focused { window: wid(2) });
    let effects = h.open(3, 0).to_vec();

    // Assert -- window 1 is laid out again.
    assert!(h.tiler.mouse.drag.is_none());
    assert!(places(&effects, 1), "{effects:?}");
}

#[test]
fn the_window_the_tiler_gave_focus_reporting_it_leaves_the_drag() {
    // Arrange -- while the user drags window 1, a key gives focus to
    // window 2.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    grab(&mut h, 1);
    h.action(Action::Focus(Direction::Right));

    // Act -- window 2 reports the focus it was given.
    h.event(WindowEvent::Focused { window: wid(2) });
    let effects = h.open(3, 0).to_vec();

    // Assert
    assert!(h.tiler.mouse.drag.is_some());
    assert!(!places(&effects, 1), "{effects:?}");
}

#[test]
fn the_dragged_window_reporting_focus_leaves_the_drag() {
    // Arrange -- window 2 has focus, and the user grabs window 1, whose
    // focus is reported after the start of its drag.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.event(WindowEvent::move_started(wid(1)));

    // Act
    h.event(WindowEvent::Focused { window: wid(1) });

    // Assert
    assert!(h.tiler.mouse.drag.is_some());
}

#[test]
fn a_dialog_of_the_window_the_tiler_gave_focus_reporting_it_leaves_the_drag() {
    // Arrange -- window 2 owns a dialog, which the system gives focus in
    // its place as a key gives focus to window 2 while the user drags
    // window 1.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    let mut dialog = FakeWindow::app(0);
    dialog.facts = None;
    dialog.owner = Some(wid(2));
    h.desktop.windows.insert(wid(9), dialog);
    grab(&mut h, 1);
    h.action(Action::Focus(Direction::Right));

    // Act
    h.event(WindowEvent::Focused { window: wid(9) });
    let effects = h.open(3, 0).to_vec();

    // Assert
    assert!(h.tiler.mouse.drag.is_some());
    assert!(!places(&effects, 1), "{effects:?}");
}
