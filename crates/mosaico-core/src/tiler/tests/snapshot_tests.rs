use super::*;
use crate::action::MAX_WORKSPACES;
use crate::id::{wid, wids};
use crate::tiler::tests::{Harness, displays, key};
use crate::tiler::{BarReservation, Input};

/// What the platform would report: only the id says which window it is.
fn seen(id: WindowId) -> WindowSnapshot {
    WindowSnapshot {
        id,
        visible: true,
        minimized: false,
        cloaked: false,
        elevated: false,
        rect: None,
        class: "App".into(),
        title: "app".into(),
    }
}

fn ids(snapshot: &DesktopSnapshot, monitor: usize, workspace: usize) -> Vec<WindowId> {
    snapshot.monitors[monitor].workspaces[workspace]
        .windows
        .iter()
        .map(|w| w.id)
        .collect()
}

#[test]
fn the_snapshot_lists_every_managed_window_by_monitor_and_workspace() {
    // Arrange -- two windows on the first monitor, one on the second.
    let h = Harness::new(2, &[(1, 0), (2, 0), (3, 1)]);

    // Act
    let snapshot = h.tiler.snapshot(true, seen);

    // Assert
    assert_eq!(snapshot.monitors.len(), 2);
    assert_eq!(ids(&snapshot, 0, 0), wids(&[1, 2]));
    assert_eq!(ids(&snapshot, 1, 0), wids(&[3]));
    let first = &snapshot.monitors[0];
    assert_eq!(first.workspaces.len(), usize::from(MAX_WORKSPACES));
    assert_eq!(first.id, h.tiler.monitors()[0].id());
    assert_eq!(first.key, key("mon0"));
    assert_eq!(first.work_area, h.tiler.monitors()[0].work_area());
    assert_eq!(first.active_workspace, 0);
    assert_eq!(snapshot.focused_monitor, h.tiler.focused_monitor());
    assert_eq!(snapshot.focused_window, h.tiler.focused_window());
    assert!(snapshot.daemon_elevated);
    assert!(snapshot.disconnected.is_empty());
}

#[test]
fn the_snapshot_reports_a_disconnected_monitor_and_its_windows() {
    // Arrange -- the second monitor goes away with its window.
    let mut h = Harness::new(2, &[(1, 0), (2, 1)]);
    h.step(Input::Displays {
        monitors: displays(1),
        bar: BarReservation::default(),
    });

    // Act
    let snapshot = h.tiler.snapshot(false, seen);

    // Assert -- the window waits on the remaining monitor.
    assert_eq!(snapshot.monitors.len(), 1);
    assert_eq!(snapshot.disconnected, vec![key("mon1")]);
    assert_eq!(snapshot.displaced_windows, 1);
    assert!(ids(&snapshot, 0, 0).contains(&wid(2)));
}
