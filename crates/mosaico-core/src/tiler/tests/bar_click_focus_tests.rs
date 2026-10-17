//! Where focus goes after a click on a bar: back to the window a monitor
//! left with, or parked on an empty workspace, where the pointer on a bar
//! counts as pointing at its monitor. Whatever a bar asks for, the pointer
//! stays where it is.

use crate::action::Origin;
use crate::config::WorkspaceMode;
use crate::id::wid;
use crate::{Action, Direction};

use super::super::tests::Harness;
use super::support::{following, moves_pointer, reserve_bar};

#[test]
fn coming_back_to_a_monitor_by_its_bar_focuses_the_window_it_left_with() {
    // Arrange -- window 3, the second on monitor 0, has focus there when
    // focus leaves it.
    let settings = following(WorkspaceMode::PerMonitor);
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (3, 0), (2, 1)]);
    h.action(Action::Focus(Direction::Right));
    assert_eq!(h.tiler.focused_window(), Some(wid(3)));
    h.action_from(Action::GoToWorkspace(1), Origin::Bar(1));

    // Act
    h.action_from(Action::GoToWorkspace(1), Origin::Bar(0));

    // Assert
    assert_eq!(h.tiler.focused_window(), Some(wid(3)));
}

#[test]
fn a_click_on_an_empty_workspace_parks_focus_on_the_clicked_monitor() {
    // Arrange -- the pointer rests on monitor 1's bar after the click.
    let mut h = Harness::with_settings(following(WorkspaceMode::PerMonitor), 2, &[(1, 0), (2, 1)]);
    reserve_bar(&mut h);
    h.action_from(Action::GoToWorkspace(5), Origin::Bar(1));
    h.desktop.cursor = Some((1920 + 50, 10));

    // Act -- the system hands focus back to the window on monitor 0.
    h.event(crate::WindowEvent::Focused { window: wid(1) });

    // Assert
    assert_eq!(h.tiler.monitors()[1].active_workspace(), 4);
    assert_eq!(h.tiler.focused_monitor(), 1);
    assert_eq!(h.tiler.focused_window(), None);
}

#[test]
fn a_pointer_on_a_bar_counts_as_pointing_at_its_monitor() {
    // Arrange -- focus is parked on monitor 1, and the user points at the
    // bar of monitor 0, whose window then reports focus.
    let mut h = Harness::with_settings(following(WorkspaceMode::PerMonitor), 2, &[(1, 0), (2, 1)]);
    reserve_bar(&mut h);
    h.action_from(Action::GoToWorkspace(5), Origin::Bar(1));
    h.desktop.cursor = Some((50, 10));

    // Act
    h.event(crate::WindowEvent::Focused { window: wid(1) });

    // Assert
    assert_eq!(h.tiler.focused_monitor(), 0);
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
}

#[test]
fn a_move_asked_for_from_a_bar_leaves_the_pointer_where_it_is() {
    // Arrange -- the pointer follows focus, and window 1 has it.
    let settings = following(WorkspaceMode::PerMonitor);
    let mut h = Harness::with_settings(settings, 1, &[(1, 0), (2, 0)]);

    // Act
    h.action_from(Action::Move(Direction::Right), Origin::Bar(0));

    // Assert -- the windows swapped places, and the pointer stayed.
    assert_eq!(h.active(0), [2, 1]);
    assert!(!moves_pointer(&h));
}
