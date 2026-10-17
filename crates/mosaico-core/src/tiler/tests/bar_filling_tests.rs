//! When the tiler tells the bars to draw again because a window on
//! screen went in or out of maximized or fullscreen, which only reports
//! that it moved.

use crate::id::wid;
use crate::tiler::tests::fake_desktop::FakeWindow;
use crate::tiler::tests::fake_platform::FakePlatform;
use crate::tiler::tests::manager_support::{start, still};
use crate::tiler::tests::{Harness, MON_H, MON_W};
use crate::{Action, Rect, WindowEvent};

/// Reports that `window` moved, and whether the bars must be drawn again.
fn moved(h: &mut Harness, window: u64) -> bool {
    h.event(WindowEvent::LocationChanged {
        window: wid(window),
    });
    h.tiler.take_bars_changed()
}

#[test]
fn a_window_on_screen_maximized_has_the_bars_drawn_again_once() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.window(2).maximized = true;

    // Act -- it reports moving again as it settles.
    let marks = [moved(&mut h, 2), moved(&mut h, 2)];

    // Assert
    assert_eq!(marks, [true, false]);
}

#[test]
fn a_window_restored_from_maximized_has_the_bars_drawn_again() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.window(2).maximized = true;
    moved(&mut h, 2);

    // Act
    h.window(2).maximized = false;
    let changed = moved(&mut h, 2);

    // Assert
    assert!(changed);
}

#[test]
fn a_window_that_moves_without_filling_its_monitor_leaves_the_bars() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0)]);
    h.window(1).rect = Rect::new(0, 0, MON_W / 2, MON_H);

    // Act
    let changed = moved(&mut h, 1);

    // Assert
    assert!(!changed);
}

#[test]
fn an_unmanaged_window_going_fullscreen_has_the_bars_drawn_again() {
    // Arrange -- window 9, which the tiler leaves alone, covers monitor 1.
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    let mut player = FakeWindow::app(1);
    player.facts = None;
    player.rect = Rect::new(MON_W, 0, MON_W, MON_H);
    h.desktop.windows.insert(wid(9), player);

    // Act
    let changed = moved(&mut h, 9);

    // Assert
    assert!(changed);
}

#[test]
fn a_maximized_window_going_to_another_monitor_has_the_bars_drawn_again() {
    // Arrange
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    h.window(1).maximized = true;
    moved(&mut h, 1);

    // Act
    h.window(1).monitor = 1;
    let changed = moved(&mut h, 1);

    // Assert
    assert!(changed);
}

#[test]
fn a_window_the_tiler_hid_leaves_the_bars() {
    // Arrange -- window 1 goes to workspace 2, and monitor 0 back to 1,
    // which leaves window 1 out of view, maximized.
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    h.window(1).maximized = true;

    // Act
    let changed = moved(&mut h, 1);

    // Assert
    assert!(!changed);
}

#[test]
fn the_manager_says_when_the_bars_must_be_drawn_again() {
    // Arrange
    let platform = FakePlatform::new(1, &[(1, 0), (2, 0)]);
    let mut manager = start(&platform, None, &still());
    platform.edit(2, |w| w.maximized = true);

    // Act
    manager.handle_event(&WindowEvent::LocationChanged { window: wid(2) });

    // Assert
    assert!(manager.take_bars_changed());
    assert!(!manager.take_bars_changed());
}

#[test]
fn a_window_hidden_or_minimized_but_still_marked_maximized_leaves_the_bars() {
    // Arrange -- windows 8 and 9, which the tiler leaves alone, say they
    // are maximized, but one is hidden and the other minimized.
    let mut h = Harness::new(1, &[(1, 0)]);
    let mut hidden = FakeWindow::app(0);
    (hidden.facts, hidden.maximized, hidden.visible) = (None, true, false);
    let mut minimized = FakeWindow::app(0);
    (minimized.facts, minimized.maximized, minimized.minimized) = (None, true, true);
    h.desktop.windows.insert(wid(8), hidden);
    h.desktop.windows.insert(wid(9), minimized);

    // Act
    let changes = [moved(&mut h, 8), moved(&mut h, 9)];

    // Assert
    assert_eq!(changes, [false, false]);
}
