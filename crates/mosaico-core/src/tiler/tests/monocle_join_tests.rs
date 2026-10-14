//! How the windows a display change brings join a monocle workspace
//! (#132).
//!
//! A workspace whose monocle window the user sent away is tiled, so a
//! window that lands there does not wait, and keeps focus. A workspace
//! whose monocle window goes home counts the windows that come in the same
//! change. On an empty monocle workspace the window that has focus becomes
//! the monocle window. The monocle window is placed before the windows
//! that wait behind it, and a hidden window is put behind once it is
//! shown.

use crate::config::HidingBehaviour;

use super::*;

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

/// Where in the last step's effects the first one `wanted` picks is.
fn first(h: &Harness, wanted: impl Fn(&Effect) -> bool) -> Option<usize> {
    h.effects.iter().position(wanted)
}

/// Where in the last step's effects the last one `wanted` picks is.
fn last(h: &Harness, wanted: impl Fn(&Effect) -> bool) -> Option<usize> {
    h.effects.iter().rposition(wanted)
}

fn put_behind(window: u64) -> impl Fn(&Effect) -> bool {
    move |e| matches!(*e, Effect::PutBehind { window: w, .. } if w == wid(window))
}

fn placed(window: u64) -> impl Fn(&Effect) -> bool {
    move |e| matches!(*e, Effect::Place { window: w, .. } if w == wid(window))
}

fn shown(window: u64) -> impl Fn(&Effect) -> bool {
    move |e| matches!(*e, Effect::Show { window: w, .. } if w == wid(window))
}

/// The primary's workspace 1 holds window 3, in monocle with window 1,
/// which the user sent to workspace 2 and left there, so workspace 1 is
/// tiled. Window 2, on the second monitor, has focus.
fn monocle_window_sent_away() -> Harness {
    let mut h = Harness::new(2, &[(1, 0), (3, 0), (2, 1)]);
    // Window 1 has focus at the start.
    h.action(Action::ToggleMonocle);
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    h.event(WindowEvent::Focused { window: wid(2) });
    h
}

#[test]
fn a_window_that_lands_where_the_monocle_window_was_sent_away_keeps_focus() {
    // Arrange
    let mut h = monocle_window_sent_away();

    // Act -- the second monitor leaves, so window 2 lands on workspace 1.
    connect_all_but(&mut h, 2, &[1]);

    // Assert -- as on any tiled workspace, window 2 keeps focus, and
    // nothing is put behind anything.
    assert!(h.workspace(0, 0).contains(wid(2)));
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
    assert_eq!(first(&h, put_behind(2)), None);
}

#[test]
fn a_window_focused_by_pointing_at_it_keeps_focus_through_a_display_change() {
    // Arrange -- window 2 lands where the monocle window was sent away,
    // and the user points at it, which focuses it.
    let mut h = monocle_window_sent_away();
    connect_all_but(&mut h, 3, &[1, 2]);
    h.event(WindowEvent::MouseHover { window: wid(2) });

    // Act -- a third monitor connects, and no window moves.
    connect_all_but(&mut h, 3, &[1]);

    // Assert
    assert_eq!(h.tiler.focused_window(), Some(wid(2)));
}

#[test]
fn an_empty_monocle_workspace_keeps_monocle_when_its_window_goes_home_as_another_comes() {
    // Arrange -- monocle is on for the primary's workspace 1, which is
    // empty. The second monitor leaves, so its window 2 becomes the
    // monocle window there.
    let mut h = Harness::new(3, &[(2, 1), (4, 2)]);
    h.action(Action::ToggleMonocle);
    connect_all_but(&mut h, 3, &[1]);

    // Act -- in one change the second monitor comes back and the third
    // leaves: window 2 goes home, and window 4 comes.
    connect_all_but(&mut h, 3, &[2]);

    // Assert -- window 4 becomes the monocle window, so monocle stays on.
    assert!(h.workspace(1, 0).contains(wid(2)));
    let ws = h.workspace(0, 0);
    assert!(ws.monocle());
    assert_eq!(ws.monocle_window(), Some(wid(4)));
    assert_eq!(h.rect_of(4), h.tiler.monocle_rect(0));
}

#[test]
fn the_window_with_focus_becomes_the_monocle_window_of_an_empty_monocle_workspace() {
    // Arrange -- monocle is on for the primary's workspace 1, which is
    // empty. Windows 2 and 3 are on the second monitor, with focus on 3.
    let mut h = Harness::new(2, &[(2, 1), (3, 1)]);
    h.action(Action::ToggleMonocle);
    h.event(WindowEvent::Focused { window: wid(3) });

    // Act -- the second monitor leaves, and both land there.
    connect_all_but(&mut h, 2, &[1]);

    // Assert -- window 3 keeps focus at the monocle rect, and window 2
    // waits behind it.
    let ws = h.workspace(0, 0);
    assert_eq!(ws.monocle_window(), Some(wid(3)));
    assert_eq!(h.tiler.focused_window(), Some(wid(3)));
    assert!(h.desktop.in_front(wid(3), wid(2)));
    assert_eq!(h.rect_of(3), h.tiler.monocle_rect(0));
    assert_eq!(h.rect_of(2), h.tiler.monocle_rect(0));
}

#[test]
fn a_monocle_window_that_comes_home_is_placed_before_the_windows_behind_it() {
    // Arrange -- the second monitor's workspace 1 holds windows 2 and 3,
    // in monocle with window 2. Both wait on the primary while it is
    // away, and the user clicks window 3 there, then window 1.
    let mut h = Harness::new(2, &[(1, 0), (2, 1), (3, 1)]);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::ToggleMonocle);
    connect_all_but(&mut h, 2, &[1]);
    h.event(WindowEvent::Focused { window: wid(3) });
    h.event(WindowEvent::Focused { window: wid(1) });

    // Act -- it comes back, and both go home.
    connect_all_but(&mut h, 2, &[]);

    // Assert -- window 2 is in place before window 3 goes behind it, so
    // window 3 never shows there alone at full size.
    let front = first(&h, placed(2)).expect("window 2 is placed");
    let behind = first(&h, put_behind(3)).expect("window 3 is put behind");
    assert!(
        front < behind,
        "window 2 is placed at {front}, after window 3 goes behind it at {behind}"
    );
}

#[test]
fn a_hidden_window_that_waits_is_put_behind_once_it_is_shown() {
    // Arrange -- with "hide", window 1 is in monocle on the primary's
    // workspace 2, in view. Window 2 is hidden on the second monitor's
    // workspace 2, which is not in view.
    let mut settings = Settings::default();
    settings.layout.hiding = HidingBehaviour::Hide;
    let mut h = Harness::with_settings(settings, 2, &[(1, 0), (2, 1)]);
    // Window 1 has focus at the start.
    h.action(Action::SendToWorkspace(2));
    h.action(Action::ToggleMonocle);
    h.event(WindowEvent::Focused { window: wid(2) });
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    assert!(!h.desktop.windows[&wid(2)].visible);

    // Act -- the second monitor leaves, and window 2 lands on the
    // primary's workspace 2, in view.
    connect_all_but(&mut h, 2, &[1]);

    // Assert -- showing a hidden window may bring it to the front, so it
    // is put behind window 1 after it is shown. The fake desktop does not
    // raise a window it shows, so the order of the effects stands in.
    let show = first(&h, shown(2)).expect("window 2 is shown");
    let behind = last(&h, put_behind(2)).expect("window 2 is put behind");
    assert!(
        show < behind,
        "window 2 is shown at {show}, after it last goes behind at {behind}"
    );
    assert!(h.desktop.in_front(wid(1), wid(2)));
}
