//! A display change that moves windows onto a monocle workspace in view
//! (#132).
//!
//! A window a display change brings from another monitor waits behind
//! the monocle window there, at the monocle rect, and monocle stays on.
//! One that brings focus with it waits behind too, and focus goes to the
//! monocle window. A monocle workspace with no window takes the newcomer
//! as its monocle window. Everyday monocle still places only the monocle
//! window. The cases out of view are in `monocle_out_of_view_tests.rs`.

use crate::config::{HidingBehaviour, WorkspaceMode};

use super::*;

fn settings(mode: WorkspaceMode, how: HidingBehaviour) -> Settings {
    let mut settings = Settings::default();
    settings.workspaces.mode = mode;
    settings.layout.hiding = how;
    settings
}

/// Connects the first `count` monitors of [`displays`].
fn connect(h: &mut Harness, count: usize) {
    h.step(Input::Displays {
        monitors: displays(count),
        bar: BarReservation::default(),
    });
}

/// Window 1 is in monocle on the primary's workspace 1, with focus, and
/// window 2 is on the second monitor's workspace 1.
fn monocle_on_the_primary(mode: WorkspaceMode) -> Harness {
    let settings = settings(mode, HidingBehaviour::Cloak);
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 1)]);
    h.action(Action::ToggleMonocle);
    h
}

/// Disconnects the second monitor, so window 2 joins the primary's
/// monocle workspace, and checks that it waits behind window 1.
fn disconnect_onto_monocle(mode: WorkspaceMode) {
    // Arrange
    let mut h = monocle_on_the_primary(mode);

    // Act
    connect(&mut h, 1);

    // Assert -- #132: window 2 used to keep the second monitor's place.
    assert_eq!(h.rect_of(2), h.tiler.monocle_rect(0));
    assert!(h.desktop.in_front(wid(1), wid(2)));
    assert!(h.workspace(0, 0).monocle());
    assert_eq!(h.workspace(0, 0).monocle_window(), Some(wid(1)));
    assert_eq!(h.rect_of(1), h.tiler.monocle_rect(0));
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
    // It goes behind before it is placed, so it never shows in front of
    // window 1 at full size.
    let at = |wanted: fn(&Effect) -> bool| h.effects.iter().position(wanted);
    let put_behind = at(|e| matches!(*e, Effect::PutBehind { window, .. } if window == wid(2)));
    let placed = at(|e| matches!(*e, Effect::Place { window, .. } if window == wid(2)));
    assert!(
        put_behind.is_some_and(|behind| placed.is_some_and(|placed| behind < placed)),
        "window 2 is put behind at {put_behind:?} and placed at {placed:?}"
    );
}

#[test]
fn a_window_moved_onto_a_monocle_workspace_in_view_waits_behind_the_monocle_window() {
    disconnect_onto_monocle(WorkspaceMode::PerMonitor);
}

#[test]
fn a_window_moved_onto_a_monocle_workspace_in_view_waits_behind_it_in_global_mode() {
    disconnect_onto_monocle(WorkspaceMode::Global);
}

#[test]
fn a_window_activated_after_the_monocle_window_still_waits_behind_it() {
    // Arrange -- window 1 is in monocle on the primary. The user then
    // works in window 2 on the second monitor, then in window 5 on the
    // third, so window 2 was in front of window 1 last.
    let mut h = Harness::new(3, &[(1, 0), (2, 1), (5, 2)]);
    h.action(Action::ToggleMonocle);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.event(WindowEvent::Focused { window: wid(5) });

    // Act -- the second monitor leaves.
    let mut monitors = displays(3);
    monitors.remove(1);
    h.step(Input::Displays {
        monitors,
        bar: BarReservation::default(),
    });

    // Assert -- #132: window 2 used to keep the second monitor's place.
    // It is put behind window 1, though focus stays on window 5.
    assert_eq!(h.rect_of(2), h.tiler.monocle_rect(0));
    assert!(h.desktop.in_front(wid(1), wid(2)));
    assert!(h.workspace(0, 0).monocle());
    assert_eq!(h.tiler.focused_window(), Some(wid(5)));
}

#[test]
fn a_window_that_brings_focus_onto_a_monocle_workspace_waits_behind_and_hands_focus_on() {
    // Arrange
    let mut h = monocle_on_the_primary(WorkspaceMode::PerMonitor);
    h.event(WindowEvent::Focused { window: wid(2) });

    // Act
    connect(&mut h, 1);

    // Assert -- #132: window 2 used to keep focus at the second
    // monitor's place, over the monocle window. It waits behind window 1
    // like any other, and focus goes to window 1, which is in front.
    assert_eq!(h.rect_of(2), h.tiler.monocle_rect(0));
    assert!(h.desktop.in_front(wid(1), wid(2)));
    assert!(h.workspace(0, 0).monocle());
    assert_eq!(h.workspace(0, 0).monocle_window(), Some(wid(1)));
    assert_eq!(h.rect_of(1), h.tiler.monocle_rect(0));
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
}

#[test]
fn a_monocle_workspace_with_no_window_takes_the_newcomer_as_its_monocle_window() {
    // Arrange -- monocle is on for the primary's workspace 1, which is
    // empty, and window 2 is on the second monitor.
    let mut h = Harness::new(2, &[(2, 1)]);
    h.action(Action::ToggleMonocle);

    // Act
    connect(&mut h, 1);

    // Assert -- #132: the workspace used to stay in monocle with no
    // monocle window. The newcomer becomes it, as on adoption.
    assert_eq!(h.workspace(0, 0).monocle_window(), Some(wid(2)));
    assert!(h.workspace(0, 0).monocle());
    assert_eq!(h.rect_of(2), h.tiler.monocle_rect(0));
}

#[test]
fn a_newcomer_waits_behind_the_window_monocle_shows_when_there_is_no_monocle_window() {
    // Arrange -- window 2 is on the second monitor's workspace 2. Monocle
    // is on for the primary's workspace 2 with no monocle window, and
    // shows window 4, which has focus there. Focus only ever moves by
    // actions here: a report that window 4 took focus would end monocle.
    let mut h = Harness::new(2, &[(4, 0), (2, 1)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::SendToWorkspace(2));
    h.action(Action::Focus(crate::Direction::Left));
    h.action(Action::GoToWorkspace(2));
    h.action(Action::ToggleMonocle);
    h.action(Action::GoToWorkspace(1));
    h.action(Action::SendToWorkspace(2));

    // Act -- window 2 moves to the primary's workspace 2, then goes home.
    connect(&mut h, 1);
    let behind = h.rect_of(2) == h.tiler.monocle_rect(0) && h.desktop.in_front(wid(4), wid(2));
    let monocle_window = h.workspace(0, 1).monocle_window();
    connect(&mut h, 2);

    // Assert -- #132: window 2 used to keep the second monitor's place.
    // It waits behind window 4 without taking over as monocle window, so
    // monocle stays when it goes home.
    assert!(behind, "window 2 waits behind window 4");
    assert_eq!(monocle_window, None);
    assert!(h.workspace(0, 1).monocle());
    assert_eq!(h.rect_of(4), h.tiler.monocle_rect(0));
}

#[test]
fn a_monocle_workspace_that_receives_no_window_stays_in_monocle() {
    // Arrange -- window 2 sits on the second monitor's workspace 2.
    let mut h = monocle_on_the_primary(WorkspaceMode::PerMonitor);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));

    // Act -- it moves to the primary's workspace 2.
    connect(&mut h, 1);

    // Assert
    assert!(h.workspace(0, 0).monocle());
    assert_eq!(h.workspace(0, 0).monocle_window(), Some(wid(1)));
    assert_eq!(h.rect_of(1), h.tiler.monocle_rect(0));
    assert!(h.workspace(0, 1).contains(wid(2)));
}

#[test]
fn a_monitor_switched_onto_its_own_monocle_workspace_keeps_monocle() {
    // Arrange -- in per-monitor mode, the primary shows workspace 1, with
    // window 2, and has windows 1 and 4 on workspace 2, in monocle with
    // window 4. The second monitor shows workspace 2, with focus. Global
    // mode is then set without lining them up, as no input can.
    let settings = settings(WorkspaceMode::PerMonitor, HidingBehaviour::Cloak);
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 0), (3, 1)]);
    // Window 1 has focus at the start.
    h.action(Action::SendToWorkspace(2));
    h.open(4, 0);
    h.action(Action::ToggleMonocle);
    h.action(Action::GoToWorkspace(1));
    h.event(WindowEvent::Focused { window: wid(3) });
    h.action(Action::GoToWorkspace(2));
    h.tiler.workspace_mode = WorkspaceMode::Global;

    // Act -- a third monitor connects, and the primary lines up on
    // workspace 2. Its windows there never left it.
    connect(&mut h, 3);

    // Assert
    assert_eq!(h.tiler.monitors()[0].active_workspace(), 1);
    assert!(h.workspace(0, 1).monocle());
    assert_eq!(h.workspace(0, 1).monocle_window(), Some(wid(4)));
    assert_eq!(h.rect_of(4), h.tiler.monocle_rect(0));
}

#[test]
fn turning_monocle_on_places_only_the_monocle_window() {
    // Arrange
    let mut h = Harness::new(1, &[(1, 0), (2, 0), (3, 0)]);

    // Act -- window 1 has focus.
    h.action(Action::ToggleMonocle);

    // Assert -- the others stay in their slots.
    let placed: Vec<u64> = h.placements().iter().map(|&(w, _, _)| w).collect();
    assert_eq!(placed, vec![1]);
}

#[test]
fn a_window_sent_to_a_monocle_workspace_is_left_where_it_was() {
    // Arrange -- window 1 is in monocle on workspace 2, and window 3 is
    // on workspace 1, with focus.
    let mut h = Harness::new(1, &[(1, 0), (3, 0)]);
    // Window 1 has focus at the start.
    h.action(Action::SendToWorkspace(2));
    h.action(Action::ToggleMonocle);
    h.action(Action::GoToWorkspace(1));
    let before = h.rect_of(3);

    // Act
    h.action(Action::SendToWorkspace(2));

    // Assert -- only the monocle window is placed, and nothing is put
    // behind it.
    let placed: Vec<u64> = h.placements().iter().map(|&(w, _, _)| w).collect();
    assert_eq!(placed, vec![1]);
    assert_eq!(h.rect_of(3), before);
    let put_behind = |e: &Effect| matches!(e, Effect::PutBehind { .. });
    assert!(!h.effects.iter().any(put_behind));
}

#[test]
fn a_window_sent_to_a_monocle_workspace_keeps_focus_through_a_display_change() {
    // Arrange -- window 1 is in monocle on workspace 2, and window 3 is
    // sent there from workspace 1, which gives it focus and leaves
    // monocle on.
    let mut h = Harness::new(1, &[(1, 0), (3, 0)]);
    // Window 1 has focus at the start.
    h.action(Action::SendToWorkspace(2));
    h.action(Action::ToggleMonocle);
    h.action(Action::GoToWorkspace(1));
    h.action(Action::SendToWorkspace(2));

    // Act -- a second monitor connects.
    connect(&mut h, 2);

    // Assert -- only a window that the change brings hands focus to the
    // monocle window.
    assert_eq!(h.tiler.focused_window(), Some(wid(3)));
    assert!(h.workspace(0, 1).monocle());
    assert_eq!(h.workspace(0, 1).monocle_window(), Some(wid(1)));
}
