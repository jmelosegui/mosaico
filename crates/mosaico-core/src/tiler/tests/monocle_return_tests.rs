//! Monocle through a disconnect and a reconnect (#132).
//!
//! Windows that come home with a returning monitor wait behind their
//! workspace's monocle window, also when they bring focus, which goes to
//! the monocle window, and a workspace keeps monocle while windows from
//! another monitor come and go, with focus or not. A monocle window that
//! goes back to its own monitor leaves no monocle behind on a workspace
//! with other windows.

use super::*;

/// Connects the first `count` monitors of [`displays`].
fn connect(h: &mut Harness, count: usize) {
    h.step(Input::Displays {
        monitors: displays(count),
        bar: BarReservation::default(),
    });
}

#[test]
fn windows_that_come_home_to_a_monocle_workspace_wait_behind_its_monocle_window() {
    // Arrange -- the second monitor's workspace 1 holds windows 2 and 3,
    // in monocle with window 2. Both wait on the primary while it is
    // away, and the user clicks window 3 there, then window 1.
    let mut h = Harness::new(2, &[(1, 0), (2, 1), (3, 1)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleMonocle);
    connect(&mut h, 1);
    h.event(WindowEvent::Focused { window: wid(3) });
    h.event(WindowEvent::Focused { window: wid(1) });

    // Act -- it comes back, and both go home.
    connect(&mut h, 2);

    // Assert -- #132: window 3 used to stay at its slot on the primary.
    assert_eq!(h.rect_of(3), h.tiler.monocle_rect(1));
    assert!(h.desktop.in_front(wid(2), wid(3)));
    assert!(h.workspace(1, 0).monocle());
    assert_eq!(h.workspace(1, 0).monocle_window(), Some(wid(2)));
    assert_eq!(h.rect_of(2), h.tiler.monocle_rect(1));
}

#[test]
fn a_window_with_focus_that_comes_home_into_view_waits_behind_and_hands_focus_on() {
    // Arrange -- in global mode, the second monitor's workspace 2 holds
    // windows 2 and 3, in monocle with window 2, while both monitors show
    // workspace 1. The second monitor leaves, and the primary switches to
    // workspace 2, where windows 2 and 3 now are, with focus on window 3.
    let mut settings = Settings::with_gap(8);
    settings.workspaces.mode = crate::config::WorkspaceMode::Global;
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 1), (3, 1)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::SendToWorkspace(2));
    h.event(WindowEvent::Focused { window: wid(3) });
    h.action(Action::SendToWorkspace(2));
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleMonocle);
    h.action(Action::GoToWorkspace(1));
    connect(&mut h, 1);
    h.action(Action::GoToWorkspace(2));
    h.event(WindowEvent::Focused { window: wid(3) });

    // Act -- the second monitor comes back on workspace 1 and lines up on
    // workspace 2, where windows 2 and 3 go home.
    connect(&mut h, 2);

    // Assert -- #132: window 3 used to stay at its place on the primary.
    // It waits behind window 2 like any other, and focus goes to window
    // 2, which is in front.
    assert_eq!(h.tiler.monitors()[1].active_workspace(), 1);
    assert_eq!(h.rect_of(3), h.tiler.monocle_rect(1));
    assert!(h.desktop.in_front(wid(2), wid(3)));
    assert!(h.workspace(1, 1).monocle());
    assert_eq!(h.workspace(1, 1).monocle_window(), Some(wid(2)));
    assert_eq!(h.rect_of(2), h.tiler.monocle_rect(1));
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
}

#[test]
fn a_monocle_workspace_keeps_monocle_while_a_window_visits() {
    // Arrange -- the primary's workspace 1 holds windows 1 and 3, in
    // monocle with window 1, which has focus. Window 2 is on the second
    // monitor.
    let mut h = Harness::new(2, &[(1, 0), (3, 0), (2, 1)]);
    h.action(Action::ToggleMonocle);

    // Act -- the second monitor goes away and comes back, as when it
    // sleeps, so window 2 visits the primary and goes home.
    connect(&mut h, 1);
    connect(&mut h, 2);

    // Assert -- window 1 is still in monocle, in front.
    assert!(h.workspace(1, 0).contains(wid(2)));
    assert!(h.workspace(0, 0).monocle());
    assert_eq!(h.workspace(0, 0).monocle_window(), Some(wid(1)));
    assert_eq!(h.rect_of(1), h.tiler.monocle_rect(0));
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
}

#[test]
fn a_monocle_workspace_keeps_monocle_while_a_window_with_focus_visits() {
    // Arrange -- as above, but window 2, on the second monitor, has
    // focus.
    let mut h = Harness::new(2, &[(1, 0), (3, 0), (2, 1)]);
    // Window 1 has focus at the start.
    h.action(Action::ToggleMonocle);
    h.event(WindowEvent::Focused { window: wid(2) });

    // Act -- window 2 visits the primary with focus and goes home.
    connect(&mut h, 1);
    connect(&mut h, 2);

    // Assert -- the primary's workspace is as it was: window 1 in
    // monocle, in front of window 3.
    assert!(h.workspace(1, 0).contains(wid(2)));
    assert!(h.workspace(0, 0).monocle());
    assert_eq!(h.workspace(0, 0).monocle_window(), Some(wid(1)));
    assert_eq!(h.rect_of(1), h.tiler.monocle_rect(0));
    assert!(h.desktop.in_front(wid(1), wid(3)));
}

#[test]
fn a_monocle_workspace_out_of_view_keeps_monocle_while_a_window_visits() {
    // Arrange -- window 1 is in monocle on the primary's workspace 2,
    // while the primary shows workspace 1, with window 3. Window 2 is on
    // the second monitor's workspace 2, which is not in view either.
    let mut h = Harness::new(2, &[(1, 0), (3, 0), (2, 1)]);
    // Window 1 has focus at the start.
    h.action(Action::SendToWorkspace(2));
    h.action(Action::ToggleMonocle);
    h.action(Action::GoToWorkspace(1));
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));

    // Act -- window 2 visits the primary's workspace 2 and goes home,
    // then the primary switches there.
    connect(&mut h, 1);
    connect(&mut h, 2);
    h.action(Action::GoToWorkspace(2));

    // Assert
    assert!(h.workspace(1, 1).contains(wid(2)));
    assert!(h.workspace(0, 1).monocle());
    assert_eq!(h.workspace(0, 1).monocle_window(), Some(wid(1)));
    assert_eq!(h.rect_of(1), h.tiler.monocle_rect(0));
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
}

#[test]
fn an_empty_monocle_workspace_keeps_monocle_while_a_window_visits() {
    // Arrange -- monocle is on for the primary's workspace 1, which is
    // empty, and window 2 is on the second monitor.
    let mut h = Harness::new(2, &[(2, 1)]);
    h.action(Action::ToggleMonocle);

    // Act -- window 2 visits the primary and goes home.
    connect(&mut h, 1);
    connect(&mut h, 2);

    // Assert -- the workspace is as it was: empty, in monocle, with no
    // monocle window.
    assert!(h.workspace(0, 0).is_empty());
    assert!(h.workspace(0, 0).monocle());
    assert_eq!(h.workspace(0, 0).monocle_window(), None);
}

#[test]
fn a_monocle_window_that_goes_home_leaves_no_monocle_behind() {
    // Arrange -- window 2 waits on the primary while the second monitor
    // is away, and is put in monocle there.
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    connect(&mut h, 1);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleMonocle);

    // Act -- the second monitor comes back, and window 2 goes home.
    connect(&mut h, 2);

    // Assert -- #132: the primary used to stay in monocle, with a
    // monocle window it no longer holds.
    assert!(!h.workspace(0, 0).monocle());
    assert_eq!(h.workspace(0, 0).monocle_window(), None);
    assert_eq!(h.active(0), vec![1]);
    assert!(h.workspace(1, 0).contains(wid(2)));
}

/// Window 1 is in monocle on the primary's workspace 2, while the
/// primary shows workspace 1, with window 3. The second monitor leaves,
/// so its window 2 lands on the primary's workspace 2, out of view, and
/// waits behind window 1 there.
fn window_2_waiting_out_of_view() -> Harness {
    let mut h = Harness::new(2, &[(1, 0), (3, 0), (2, 1)]);
    // Window 1 has focus at the start.
    h.action(Action::SendToWorkspace(2));
    h.action(Action::ToggleMonocle);
    h.action(Action::GoToWorkspace(1));
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    connect(&mut h, 1);
    h
}

/// The windows the last step placed, in order.
fn placed(h: &Harness) -> Vec<u64> {
    h.placements().iter().map(|&(w, _, _)| w).collect()
}

#[test]
fn a_window_that_closes_while_it_waits_leaves_nothing_waiting_for_its_id() {
    // Arrange -- window 2 closes while it waits, and the primary switches
    // to the monocle workspace.
    let mut h = window_2_waiting_out_of_view();
    h.desktop.windows.remove(&wid(2));
    h.event(WindowEvent::Destroyed { window: wid(2) });
    h.action(Action::GoToWorkspace(2));

    // Act -- a new window gets its id there, and is taken in when it
    // reports focus.
    h.desktop.windows.insert(wid(2), FakeWindow::app(0));
    h.event(WindowEvent::Focused { window: wid(2) });

    // Assert -- as for any window taken in there that way, only the
    // monocle window is placed, and nothing is put behind it.
    assert!(h.workspace(0, 1).contains(wid(2)));
    assert_eq!(placed(&h), vec![1]);
    let put_behind = |e: &Effect| matches!(e, Effect::PutBehind { .. });
    assert!(!h.effects.iter().any(put_behind));
}

#[test]
fn a_window_tiled_after_it_waited_is_left_where_it_is_by_a_later_monocle() {
    // Arrange -- the monocle window closes while window 2 waits, which
    // ends monocle there, and the primary switches there, so window 2 is
    // tiled. A new window 5 joins it, with focus.
    let mut h = window_2_waiting_out_of_view();
    h.desktop.windows.remove(&wid(1));
    h.event(WindowEvent::Destroyed { window: wid(1) });
    h.action(Action::GoToWorkspace(2));
    h.open(5, 0);

    // Act
    h.action(Action::ToggleMonocle);

    // Assert -- only window 5, the new monocle window, is placed.
    assert_eq!(placed(&h), vec![5]);
}
