//! What a display change leaves behind on monocle workspaces (#132).
//!
//! A window that waited behind a monocle window leaves nothing waiting
//! for it once it is off that workspace, whether it went home, was let go
//! by the rules or closed. A display change only settles monocle on the
//! workspaces it took the monocle window from: a returning monitor's that
//! did not get it back leaves monocle, and one whose monocle window went
//! home hands monocle to the window in front of those that wait there.

use crate::config::WindowRule;

use super::*;

/// Connects the first `count` monitors of [`displays`].
fn connect(h: &mut Harness, count: usize) {
    h.step(Input::Displays {
        monitors: displays(count),
        bar: BarReservation::default(),
    });
}

/// Connects the first `count` monitors of [`displays`], except the ones
/// at the indices in `away`.
fn connect_all_but(h: &mut Harness, count: usize, away: &[usize]) {
    let monitors = displays(count)
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !away.contains(i))
        .map(|(_, display)| display)
        .collect();
    h.step(Input::Displays {
        monitors,
        bar: BarReservation::default(),
    });
}

/// The windows the last step placed, in order.
fn placed(h: &Harness) -> Vec<u64> {
    h.placements().iter().map(|&(w, _, _)| w).collect()
}

/// Whether the last step put any window behind another.
fn put_any_behind(h: &Harness) -> bool {
    h.effects
        .iter()
        .any(|e| matches!(e, Effect::PutBehind { .. }))
}

#[test]
fn a_window_that_came_home_as_its_monocle_window_is_not_put_behind_when_dragged() {
    // Arrange -- the primary's workspace 1 is in monocle with window 1,
    // and its workspace 2 with window 5. The second monitor's workspace 2
    // is in monocle with window 2. Both monitors show workspace 1.
    let mut h = Harness::new(2, &[(1, 0), (5, 0), (2, 1)]);
    // Window 1 has focus at the start.
    h.event(WindowEvent::Focused { window: wid(5) });
    h.action(Action::SendToWorkspace(2));
    h.action(Action::ToggleMonocle);
    h.action(Action::GoToWorkspace(1));
    h.action(Action::ToggleMonocle);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::SendToWorkspace(2));
    h.action(Action::ToggleMonocle);
    h.action(Action::GoToWorkspace(1));
    // The second monitor sleeps and wakes. Window 2 waits behind window 5
    // on the primary meanwhile, and comes home as its monocle window,
    // where the user picks it from the taskbar.
    connect(&mut h, 1);
    connect(&mut h, 2);
    h.event(WindowEvent::Focused { window: wid(2) });

    // Act -- the user drags it onto the primary.
    h.window(2).monitor = 0;
    h.window(2).rect = Rect::new(200, 200, 640, 480);
    h.event(WindowEvent::Moved { window: wid(2) });

    // Assert -- as for any window dragged onto a monocle workspace, only
    // the monocle window is placed, and nothing is put behind it.
    assert!(h.workspace(0, 0).contains(wid(2)));
    assert_eq!(placed(&h), vec![1]);
    assert!(!put_any_behind(&h));
}

#[test]
fn a_window_the_rules_let_go_while_it_waits_leaves_nothing_waiting_for_its_id() {
    // Arrange -- window 1 is in monocle on the primary's workspace 2,
    // while the primary shows workspace 1, with window 3. The second
    // monitor leaves, so its window 2 lands on the primary's workspace 2,
    // out of view, and waits behind window 1 there.
    let mut h = Harness::new(2, &[(1, 0), (3, 0), (2, 1)]);
    // Window 1 has focus at the start.
    h.action(Action::SendToWorkspace(2));
    h.action(Action::ToggleMonocle);
    h.action(Action::GoToWorkspace(1));
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    connect(&mut h, 1);
    // The rules let go of it, then it closes, and the primary switches to
    // the monocle workspace.
    if let Some(facts) = h.window(2).facts.as_mut() {
        facts.class = "Excluded".into();
    }
    h.step(Input::Rules(vec![WindowRule {
        match_class: Some("Excluded".into()),
        match_title: None,
        match_path: None,
        manage: false,
    }]));
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
    assert!(!put_any_behind(&h));
}

#[test]
fn a_display_change_leaves_a_stale_monocle_alone_where_it_took_no_window() {
    // Arrange -- the primary's workspace 1 holds windows 1 and 3, in
    // monocle with window 1, which the user then sends to workspace 2.
    // Workspace 1 stays in monocle, for window 1 to come back to.
    let mut h = Harness::new(1, &[(1, 0), (3, 0)]);
    // Window 1 has focus at the start.
    h.action(Action::ToggleMonocle);
    h.action(Action::SendToWorkspace(2));

    // Act -- a second monitor connects, then window 1 is sent back.
    connect(&mut h, 2);
    h.action(Action::SendToWorkspace(1));

    // Assert -- monocle resumes with window 1, as without the display
    // change.
    assert!(h.workspace(0, 0).monocle());
    assert_eq!(h.workspace(0, 0).monocle_window(), Some(wid(1)));
    assert_eq!(h.rect_of(1), h.tiler.monocle_rect(0));
}

#[test]
fn an_empty_monocle_workspace_stays_in_monocle_through_overlapping_visits() {
    // Arrange -- monocle is on for the primary's workspace 1, which is
    // empty. The second monitor leaves, so its window 2 becomes the
    // monocle window there, then the third, so its windows 4 and 5 wait
    // behind window 2.
    let mut h = Harness::new(3, &[(2, 1), (4, 2), (5, 2)]);
    h.action(Action::ToggleMonocle);
    connect_all_but(&mut h, 3, &[1]);
    connect_all_but(&mut h, 3, &[1, 2]);

    // Act -- the second monitor comes back, and window 2 goes home.
    connect_all_but(&mut h, 3, &[2]);

    // Assert -- #132: windows 4 and 5 used to be tiled side by side
    // there, though the workspace was in monocle. Monocle stays on, with
    // the one of them in front, and the other behind it.
    assert_eq!(h.rect_of(4), h.tiler.monocle_rect(0));
    assert_eq!(h.rect_of(5), h.tiler.monocle_rect(0));
    let ws = h.workspace(0, 0);
    assert_eq!(ws.windows(), [wid(4), wid(5)]);
    assert!(ws.monocle());
    let front = ws.monocle_window().expect("a monocle window");
    let behind = if front == wid(4) { wid(5) } else { wid(4) };
    assert!(h.desktop.in_front(front, behind));
}

#[test]
fn a_returning_monitor_leaves_monocle_when_its_monocle_window_stayed_away() {
    // Arrange -- the second monitor's workspace 1 is in monocle with
    // window 2, its only window. While the monitor is away, window 2
    // waits on the primary, and the user sends it to workspace 3 there.
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleMonocle);
    connect(&mut h, 1);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::SendToWorkspace(3));

    // Act -- the second monitor comes back, without window 2.
    connect(&mut h, 2);

    // Assert -- as on the base, its workspace leaves monocle. Only a
    // workspace a window visited stays in monocle when it is left empty.
    let ws = h.workspace(1, 0);
    assert!(h.workspace(0, 2).contains(wid(2)));
    assert!(ws.is_empty());
    assert!(!ws.monocle());
    assert_eq!(ws.monocle_window(), None);
}
