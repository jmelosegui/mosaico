//! Which windows count for the bars, managed or not: those on screen,
//! wherever focus is, but neither the desktop nor a window the tiler
//! hid.

use crate::id::wid;
use crate::tiler::Desktop;
use crate::tiler::tests::fake_desktop::FakeWindow;
use crate::tiler::tests::{Harness, MON_H, MON_W};
use crate::{Action, Rect, WindowEvent};

use super::maximized_tests::marked;

/// A window the tiler leaves alone, on `monitor`.
fn unmanaged(monitor: usize) -> FakeWindow {
    let mut window = FakeWindow::app(monitor);
    window.facts = None;
    window
}

#[test]
fn an_unmanaged_maximized_window_counts_on_its_monitor_without_focus() {
    // Arrange -- window 9, say an elevated terminal, is maximized on
    // monitor 1, and focus is on monitor 0.
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    let mut terminal = unmanaged(1);
    terminal.maximized = true;
    h.desktop.windows.insert(wid(9), terminal);

    // Act
    let marks = marked(&h);

    // Assert
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
    assert_eq!(marks, [false, true]);
}

#[test]
fn the_desktop_taking_focus_marks_no_bar() {
    // Arrange -- the desktop spans both monitors, and takes focus after a
    // click on the wallpaper.
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    let mut desktop = unmanaged(0);
    desktop.shell = true;
    desktop.rect = Rect::new(0, 0, MON_W * 2, MON_H);
    h.desktop.windows.insert(wid(9), desktop);
    h.event(WindowEvent::Focused { window: wid(9) });

    // Act
    let marks = marked(&h);

    // Assert
    assert_eq!(marks, [false, false]);
}

#[test]
fn a_maximized_window_on_a_workspace_off_screen_does_not_count() {
    // Arrange -- window 1 goes to workspace 2, and monitor 0 back to 1.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    h.window(1).maximized = true;

    // Act
    let marks = marked(&h);

    // Assert
    assert_eq!(marks, [false]);
}

#[test]
fn a_maximized_window_the_tiler_hid_does_not_count_even_listed_on_screen() {
    // Arrange -- window 1, maximized, has focus when its monitor goes to
    // an empty workspace. The desktop still lists it on screen, the way a
    // window kept out of view in place could be.
    let mut h = Harness::new(1, &[(1, 0)]);
    h.window(1).maximized = true;
    h.action(Action::GoToWorkspace(2));
    h.window(1).cloaked = false;

    // Act
    let marks = marked(&h);

    // Assert
    assert_eq!(marks, [false]);
}

#[test]
fn a_managed_window_maximized_on_another_monitor_counts_there() {
    // Arrange -- window 2, tiled on monitor 1, was moved to monitor 0
    // maximized, without a drag, so the tiler still has it on monitor 1.
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    h.window(2).monitor = 0;
    h.window(2).maximized = true;

    // Act
    let marks = marked(&h);

    // Assert
    assert_eq!(marks, [true, false]);
}

#[test]
fn a_maximized_window_in_front_of_a_tiled_one_marks_their_monitor() {
    // Arrange -- window 2, maximized, comes to the front, so the desktop
    // lists window 1, tiled on the same monitor, after it.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.window(2).maximized = true;
    h.desktop.raise(wid(2));

    // Act
    let marks = marked(&h);

    // Assert
    assert_eq!(h.desktop.windows_on_screen(), [wid(2), wid(1)]);
    assert_eq!(marks, [true]);
}
