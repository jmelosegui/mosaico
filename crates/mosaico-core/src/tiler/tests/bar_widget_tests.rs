//! What a click on a widget of another monitor's bar asks of the tiler:
//! the layout and monocle on that monitor, which takes focus first, a
//! retile everywhere with focus left where it is, and never a pointer
//! move.

use crate::action::Origin;
use crate::config::WorkspaceMode;
use crate::id::wid;
use crate::{Action, LayoutKind};

use super::super::tests::Harness;
use super::support::{focused, following, moves_pointer};

/// Window 1 on monitor 0, which has focus, and windows 2 and 3 on
/// monitor 1, the pointer following focus, workspaces switched in `mode`.
fn two_monitors(mode: WorkspaceMode) -> Harness {
    let h = Harness::with_settings(following(mode), 2, &[(1, 0), (2, 1), (3, 1)]);
    assert_eq!(h.tiler.focused_monitor(), 0);
    h
}

/// The layout of the workspace on screen on monitor `mon` of `h`.
fn layout(h: &Harness, mon: usize) -> LayoutKind {
    h.tiler.monitors()[mon].active().layout_kind()
}

#[test]
fn a_layout_click_on_another_monitors_bar_cycles_that_monitors_layout() {
    // Arrange
    let mut h = two_monitors(WorkspaceMode::PerMonitor);

    // Act
    h.action_from(Action::CycleLayout, Origin::Bar(1));

    // Assert -- focus moved there first, as if the user had gone there.
    assert_eq!(layout(&h, 1), LayoutKind::Bsp.next());
    assert_eq!(layout(&h, 0), LayoutKind::Bsp);
    assert_eq!(h.tiler.focused_monitor(), 1);
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
    assert!(!moves_pointer(&h));
}

#[test]
fn the_layout_key_cycles_the_focused_monitors_layout_instead() {
    // Arrange -- the near miss: only the origin names the monitor.
    let mut h = two_monitors(WorkspaceMode::PerMonitor);

    // Act
    h.action(Action::CycleLayout);

    // Assert
    assert_eq!(layout(&h, 0), LayoutKind::Bsp.next());
    assert_eq!(layout(&h, 1), LayoutKind::Bsp);
}

#[test]
fn in_global_mode_a_layout_click_still_cycles_the_clicked_monitor_alone() {
    // Arrange -- global mode switches workspaces together, not layouts.
    let mut h = two_monitors(WorkspaceMode::Global);

    // Act
    h.action_from(Action::CycleLayout, Origin::Bar(1));

    // Assert
    assert_eq!(layout(&h, 1), LayoutKind::Bsp.next());
    assert_eq!(layout(&h, 0), LayoutKind::Bsp);
    assert_eq!(h.tiler.focused_monitor(), 1);
}

#[test]
fn a_monocle_click_on_another_monitors_bar_puts_its_window_in_front() {
    // Arrange
    let mut h = two_monitors(WorkspaceMode::PerMonitor);

    // Act
    h.action_from(Action::ToggleMonocle, Origin::Bar(1));

    // Assert -- the window that took focus there, not the one focus left.
    let (first, second) = (h.workspace(0, 0), h.workspace(1, 0));
    assert!(second.monocle() && !first.monocle());
    assert_eq!(second.monocle_window(), Some(wid(2)));
    assert!(!moves_pointer(&h));
}

#[test]
fn a_retile_click_on_another_monitors_bar_leaves_focus_where_it_is() {
    // Arrange -- retiling acts on every monitor, so none takes focus.
    let mut h = two_monitors(WorkspaceMode::PerMonitor);

    // Act
    h.action_from(Action::Retile, Origin::Bar(1));

    // Assert
    assert_eq!(h.tiler.focused_monitor(), 0);
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
    assert!(focused(&h).is_empty());
    assert!(!moves_pointer(&h));
}
