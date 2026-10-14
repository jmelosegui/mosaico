//! Where focus goes when the monitors change (#129).
//!
//! A focused window that is still in view keeps focus wherever it moved.
//! Otherwise focus stays on the same physical monitor while it is
//! connected, followed through its pairing, and goes to the primary
//! monitor, where the windows of a monitor that left go, when it is gone.
//! When the focused window lands out of view, the focused monitor's
//! window is focused as in a workspace switch, so the system's own pick
//! once the window is hidden is not followed.

use std::time::Duration;

use crate::EdidIdentity;
use crate::config::Config;

use super::fake_platform::FakePlatform;
use super::manager_support::start;
use super::*;

/// Applies a display change to `monitors`.
fn change(h: &mut Harness, monitors: Vec<Display>) {
    h.step(Input::Displays {
        monitors,
        bar: BarReservation::default(),
    });
}

/// Three monitors, the first one primary, with monitor `gone` removed.
fn three_without(gone: usize) -> Vec<Display> {
    let mut monitors = displays(3);
    monitors.remove(gone);
    monitors
}

/// Three monitors with window 1 on monitor 1 and window 2 on monitor 2.
/// Monitor `focused` has focus, on a workspace with no window, so focus
/// cannot follow a window when the monitors change.
fn focus_on_empty_workspace_of(focused: u64) -> Harness {
    let mut h = Harness::new(3, &[(1, 1), (2, 2)]);
    h.event(WindowEvent::Focused {
        window: wid(focused),
    });
    h.action(Action::GoToWorkspace(2));
    h
}

/// `count` monitors, with window 1 on monitor 1. Monitor 1 comes back
/// under a new id as one the platform cannot identify, still paired by
/// its place on the desktop, and gets focus on a workspace with no
/// window. Returns the monitors as they are now.
fn unidentified_monitor_1_with_focus(count: usize) -> (Harness, Vec<Display>) {
    let mut h = Harness::new(count, &[(1, 1)]);
    let mut monitors = displays(count);
    monitors[1].key = MonitorKey::default();
    monitors[1].id = MonitorId::from_raw(4);
    change(&mut h, monitors.clone());
    h.event(WindowEvent::Focused { window: wid(1) });
    h.action(Action::GoToWorkspace(2));
    (h, monitors)
}

#[test]
fn disconnecting_the_focused_monitor_moves_focus_to_the_primary() {
    // Arrange
    let mut h = focus_on_empty_workspace_of(1);

    // Act
    change(&mut h, three_without(1));

    // Assert -- #129: focus used to go to monitor 2, which took monitor
    // 1's place in the list.
    assert_eq!(h.tiler.focused_monitor(), 0);
    assert_eq!(h.active(0), vec![1]);
}

#[test]
fn disconnecting_the_focused_monitor_focuses_the_primary_when_its_window_is_hidden() {
    // Arrange -- window 1 on monitor 1 has focus, and the primary shows
    // workspace 2, with window 3.
    let mut h = Harness::new(3, &[(3, 0), (1, 1), (2, 2)]);
    // Window 3 has focus at the start.
    h.action(Action::SendToWorkspace(2));
    h.event(WindowEvent::Focused { window: wid(1) });

    // Act -- window 1 moves to the primary's workspace 1, out of view.
    change(&mut h, three_without(1));

    // Assert -- #129: focus used to go to window 2, on monitor 2.
    assert_eq!(h.tiler.focused_monitor(), 0);
    assert_eq!(h.tiler.focused_window(), Some(wid(3)));
    assert!(h.workspace(0, 0).contains(wid(1)));
}

#[test]
fn the_systems_own_pick_does_not_take_focus_from_the_window_chosen() {
    // Arrange -- the primary shows workspace 2, with windows 3 and 5,
    // and remembers window 3 there. Window 1 on monitor 1 has focus.
    let platform = FakePlatform::new(2, &[(3, 0), (5, 0), (1, 1)]);
    let mut manager = start(&platform, None, &Config::default());
    // Window 3 has focus at the start.
    manager.handle_action(&Action::SendToWorkspace(2));
    manager.handle_action(&Action::GoToWorkspace(1));
    manager.handle_action(&Action::SendToWorkspace(2));
    // The user clicks window 1 once the switches have landed, so its
    // report is not taken for theirs.
    manager.flush_pending();
    platform.advance(FOCUS_ECHO_TTL);
    manager.handle_event(&WindowEvent::Focused { window: wid(1) });

    // Act -- window 1 moves to the primary's workspace 1, out of view,
    // and the system hands focus to window 5 once it is hidden.
    manager.handle_display_change(displays(1), 0, &[]);
    manager.handle_event(&WindowEvent::Focused { window: wid(5) });
    manager.flush_pending();

    // Assert -- #129: window 3 used to be taken as focused without the
    // system being asked, so its own pick of window 5 was followed.
    assert_eq!(manager.tiler().focused_window(), Some(wid(3)));
    assert_eq!(platform.focused(), Some(wid(3)));
}

#[test]
fn the_monocle_window_takes_focus_from_a_window_that_lands_out_of_view() {
    // Arrange -- the primary shows workspace 2, with windows 4 and 3, in
    // monocle with window 3, and remembers window 4 there. Window 1 on
    // monitor 1 has focus.
    let mut h = Harness::new(2, &[(4, 0), (3, 0), (1, 1)]);
    // Window 4 has focus at the start.
    h.action(Action::SendToWorkspace(2));
    h.action(Action::GoToWorkspace(1));
    h.action(Action::SendToWorkspace(2));
    h.action(Action::ToggleMonocle);
    h.event(WindowEvent::Focused { window: wid(1) });

    // Act -- window 1 moves to the primary's workspace 1, out of view.
    change(&mut h, displays(1));

    // Assert -- #129: focus used to be taken as window 4's, in front of
    // the monocle window, without the system being asked. As in a
    // workspace switch, the monocle window takes it, and monocle stays.
    assert_eq!(h.tiler.focused_window(), Some(wid(3)));
    let focused_3 = |e: &Effect| matches!(*e, Effect::Focus { window, .. } if window == wid(3));
    assert!(h.effects.iter().any(focused_3));
    assert!(h.workspace(0, 1).monocle());
    assert_eq!(h.workspace(0, 1).monocle_window(), Some(wid(3)));
}

#[test]
fn focus_stays_on_the_primary_when_its_workspace_in_view_is_empty() {
    // Arrange -- monitor 0 shows an empty workspace 2. Window 1 on
    // monitor 1 has focus, and window 2 is on monitor 2.
    let mut h = Harness::new(3, &[(1, 1), (2, 2)]);
    h.action(Action::GoToWorkspace(2));
    h.advance(Duration::from_secs(60));
    h.event(WindowEvent::Focused { window: wid(1) });

    // Act -- monitor 1 leaves, so window 1 moves to the primary's
    // workspace 1, out of view, and the system hands focus to window 2
    // once it is hidden.
    change(&mut h, three_without(1));
    let after_change = h.tiler.focused_monitor();
    h.event(WindowEvent::Focused { window: wid(2) });

    // Assert -- #129: focus used to go to monitor 2. Nothing can hold it
    // on the primary, so it is parked there, and the system's pick is
    // not followed.
    assert_eq!(after_change, 0);
    assert_eq!(h.tiler.focused_monitor(), 0);
    assert_eq!(h.tiler.focused_window(), None);
}

#[test]
fn an_unidentified_monitor_that_leaves_sends_focus_to_the_primary() {
    // Arrange
    let (mut h, mut monitors) = unidentified_monitor_1_with_focus(3);

    // Act -- it disconnects.
    monitors.remove(1);
    change(&mut h, monitors);

    // Assert -- #129: focus used to go to monitor 2, now second in the
    // list, while window 1 went to the primary.
    assert_eq!(h.tiler.focused_monitor(), 0);
    assert!(h.workspace(0, 0).contains(wid(1)));
}

#[test]
fn an_unidentified_monitor_that_cannot_be_recognised_sends_focus_to_the_primary() {
    // Arrange
    let (mut h, mut monitors) = unidentified_monitor_1_with_focus(2);

    // Act -- it gets a new id and moves, so it cannot be told from a new
    // monitor.
    monitors[1].id = MonitorId::from_raw(9);
    monitors[1].work_area.x += 640;
    change(&mut h, monitors);

    // Assert -- #129: focus used to stay at its place in the list, on
    // the new monitor, while window 1 went to the primary.
    assert_eq!(h.tiler.focused_monitor(), 0);
    assert!(h.workspace(0, 0).contains(wid(1)));
}

#[test]
fn a_focused_window_that_moves_into_view_keeps_focus() {
    // Arrange
    let mut h = Harness::new(3, &[(1, 1), (2, 2)]);
    h.event(WindowEvent::Focused { window: wid(1) });

    // Act -- window 1 moves to the primary's workspace 1, in view.
    change(&mut h, three_without(1));

    // Assert
    assert_eq!(h.tiler.focused_monitor(), 0);
    assert_eq!(h.tiler.focused_window(), Some(wid(1)));
}

#[test]
fn removing_another_monitor_keeps_focus_on_its_monitor() {
    // Arrange
    let mut h = focus_on_empty_workspace_of(2);

    // Act
    change(&mut h, three_without(1));

    // Assert -- monitor 2 is the second one in the list now.
    assert_eq!(h.tiler.focused_monitor(), 1);
}

#[test]
fn an_unidentified_monitor_that_stays_keeps_focus() {
    // Arrange
    let (mut h, mut monitors) = unidentified_monitor_1_with_focus(2);

    // Act -- it gets a new id again, but keeps its place on the desktop.
    monitors[1].id = MonitorId::from_raw(5);
    change(&mut h, monitors);

    // Assert -- it is paired by its place, so it keeps focus and its
    // windows.
    assert_eq!(h.tiler.focused_monitor(), 1);
    assert!(h.workspace(1, 0).contains(wid(1)));
}

#[test]
fn a_monitor_that_reports_another_key_keeps_focus() {
    // Arrange -- monitor 1, which also has an EDID, has focus.
    let mut h = Harness::new(2, &[(1, 1)]);
    let edid = EdidIdentity {
        manufacturer: "DEL".into(),
        product: 1,
        serial: Some(7),
    };
    let mut monitors = displays(2);
    monitors[1].key.edid = Some(edid.clone());
    monitors[1].id = MonitorId::from_raw(5);
    change(&mut h, monitors.clone());
    h.event(WindowEvent::Focused { window: wid(1) });
    h.action(Action::GoToWorkspace(2));

    // Act -- it stays connected under a new id, but reports only its
    // EDID model, as the stand-in a KVM switch leaves can.
    monitors[1].key = MonitorKey {
        edid: Some(EdidIdentity {
            serial: None,
            ..edid
        }),
        ..MonitorKey::default()
    };
    monitors[1].id = MonitorId::from_raw(6);
    change(&mut h, monitors);

    // Assert -- it is the same monitor, with its workspaces.
    assert_eq!(h.tiler.focused_monitor(), 1);
    assert!(h.workspace(1, 0).contains(wid(1)));
}
