//! Floating windows leaving, coming back and following their workspace.

use crate::id::wid;
use crate::tiler::tests::Harness;
use crate::{Action, WindowEvent};

/// Windows 1 and 2 on one monitor, window 2 floated with focus.
fn floated() -> Harness {
    let mut h = Harness::new(1, &[(1, 0), (2, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleFloat);
    h
}

#[test]
fn a_floating_window_minimized_and_restored_floats_again_where_it_was() {
    // Arrange
    let mut h = floated();
    let floated = h.rect_of(2);

    // Act
    h.window(2).minimized = true;
    h.event(WindowEvent::Minimized { window: wid(2) });
    let gone = h.workspace(0, 0).floating().is_empty();
    h.window(2).minimized = false;
    h.event(WindowEvent::Restored { window: wid(2) });

    // Assert
    assert!(gone);
    assert_eq!(h.workspace(0, 0).floating(), &[wid(2)]);
    assert_eq!(h.active(0), vec![1]);
    assert_eq!(h.rect_of(2), floated);
}

#[test]
fn a_floating_window_hides_and_shows_with_its_workspace() {
    // Arrange
    let mut h = floated();

    // Act
    h.action(Action::GoToWorkspace(2));
    let hidden = h.window(2).cloaked;
    h.action(Action::GoToWorkspace(1));

    // Assert
    assert!(hidden);
    assert!(!h.window(2).cloaked);
    assert_eq!(h.focus_request(), Some(2));
}

#[test]
fn a_floating_window_is_not_sent_to_another_workspace() {
    // Arrange
    let mut h = floated();

    // Act
    h.action(Action::SendToWorkspace(2));

    // Assert
    assert_eq!(h.workspace(0, 0).floating(), &[wid(2)]);
    assert!(h.workspace(0, 1).floating().is_empty());
    assert!(!h.window(2).cloaked);
}

#[test]
fn coming_back_to_a_workspace_focuses_its_frontmost_floating_window() {
    // Arrange -- the user clicked tile 1 behind floating window 2.
    let mut h = floated();
    h.event(WindowEvent::Focused { window: wid(1) });
    h.action(Action::GoToWorkspace(2));

    // Act
    h.action(Action::GoToWorkspace(1));

    // Assert
    assert_eq!(h.focus_request(), Some(2));
}

#[test]
fn closing_a_floating_window_hands_focus_to_the_next_one() {
    // Arrange -- windows 2 and 3 float, 3 in front with focus.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    for id in [2, 3] {
        h.event(WindowEvent::Focused { window: wid(id) });
        h.action(Action::ToggleFloat);
    }
    h.desktop.windows.remove(&wid(3));

    // Act
    h.event(WindowEvent::Destroyed { window: wid(3) });

    // Assert
    assert_eq!(h.focus_request(), Some(2));
}

#[test]
fn closing_the_last_floating_window_hands_focus_to_a_tile() {
    // Arrange
    let mut h = floated();
    h.desktop.windows.remove(&wid(2));

    // Act
    h.event(WindowEvent::Destroyed { window: wid(2) });

    // Assert
    assert!(h.workspace(0, 0).floating().is_empty());
    assert_eq!(h.focus_request(), Some(1));
}

#[test]
fn monocle_ending_as_its_window_closes_shows_the_floating_windows() {
    // Arrange -- tile 1 in monocle hides floating window 2.
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleFloat);
    h.event(WindowEvent::Focused { window: wid(1) });
    h.action(Action::ToggleMonocle);
    h.desktop.windows.remove(&wid(1));

    // Act
    h.event(WindowEvent::Destroyed { window: wid(1) });

    // Assert
    assert!(!h.workspace(0, 0).monocle());
    assert!(!h.window(2).cloaked);
}
