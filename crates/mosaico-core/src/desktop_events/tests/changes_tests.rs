use super::support::*;

#[test]
fn the_same_desktop_has_no_changes() {
    // Act / Assert
    assert_eq!(changes(&desktop(), &desktop()), vec![]);
}

#[test]
fn a_new_window_is_managed() {
    // Arrange
    let mut after = desktop();
    after.monitors[0].workspaces[1].windows.push(window(B, "b"));

    // Act
    let events = changes(&desktop(), &after);

    // Assert
    assert_eq!(
        events,
        vec![DesktopEvent::WindowManaged {
            window: wid(B),
            monitor: mid(1),
            workspace: 1,
            class: "Notepad".into(),
            title: "b".into(),
        }]
    );
}

#[test]
fn a_window_that_is_gone_is_unmanaged_and_focus_moves() {
    // Arrange
    let mut after = desktop();
    after.monitors[0].workspaces[0].windows.clear();
    after.focused_window = None;

    // Act
    let events = changes(&desktop(), &after);

    // Assert
    assert_eq!(
        events,
        vec![
            DesktopEvent::WindowUnmanaged { window: wid(A) },
            DesktopEvent::FocusChanged {
                window: None,
                monitor: mid(1),
            },
        ]
    );
}

#[test]
fn a_window_sent_to_another_workspace_moved() {
    // Arrange
    let mut after = desktop();
    let a = after.monitors[0].workspaces[0].windows.remove(0);
    after.monitors[0].workspaces[2].windows.push(a);

    // Act
    let events = changes(&desktop(), &after);

    // Assert
    assert_eq!(
        events,
        vec![DesktopEvent::WindowMoved {
            window: wid(A),
            monitor: mid(1),
            workspace: 2,
        }]
    );
}

#[test]
fn a_window_that_floats_reports_it_without_moving() {
    // Arrange
    let mut after = desktop();
    let a = after.monitors[0].workspaces[0].windows.remove(0);
    after.monitors[0].workspaces[0].floating.push(a);

    // Act
    let events = changes(&desktop(), &after);

    // Assert
    assert_eq!(
        events,
        vec![DesktopEvent::WindowFloatingChanged {
            window: wid(A),
            floating: true,
        }]
    );
}

#[test]
fn a_renamed_window_reports_its_title() {
    // Arrange
    let mut after = desktop();
    after.monitors[0].workspaces[0].windows[0].title = "a*".into();

    // Act
    let events = changes(&desktop(), &after);

    // Assert
    assert_eq!(
        events,
        vec![DesktopEvent::WindowTitleChanged {
            window: wid(A),
            title: "a*".into(),
        }]
    );
}

#[test]
fn switching_workspace_reports_the_new_one() {
    // Arrange
    let mut after = desktop();
    after.monitors[0].active_workspace = 2;

    // Act
    let events = changes(&desktop(), &after);

    // Assert
    assert_eq!(
        events,
        vec![DesktopEvent::WorkspaceChanged {
            monitor: mid(1),
            workspace: 2,
        }]
    );
}

#[test]
fn a_layout_or_monocle_change_reports_the_workspace() {
    // Arrange
    let mut after = desktop();
    after.monitors[0].workspaces[0].monocle = true;
    after.monitors[0].workspaces[1].layout = LayoutKind::ThreeColumn;

    // Act
    let events = changes(&desktop(), &after);

    // Assert
    assert_eq!(
        events,
        vec![
            DesktopEvent::LayoutChanged {
                monitor: mid(1),
                workspace: 0,
                layout: LayoutKind::Bsp,
                monocle: true,
            },
            DesktopEvent::LayoutChanged {
                monitor: mid(1),
                workspace: 1,
                layout: LayoutKind::ThreeColumn,
                monocle: false,
            },
        ]
    );
}

#[test]
fn a_new_monitor_reports_itself_not_its_workspaces() {
    // Arrange
    let mut after = desktop();
    after.monitors.push(monitor(2));

    // Act
    let events = changes(&desktop(), &after);

    // Assert
    assert_eq!(events, vec![DesktopEvent::MonitorAdded { monitor: mid(2) }]);
}

#[test]
fn a_monitor_that_goes_away_takes_its_windows_to_another() {
    // Arrange
    let mut before = desktop();
    let mut second = monitor(2);
    second.workspaces[0].windows.push(window(B, "b"));
    before.monitors.push(second);
    let mut after = desktop();
    after.monitors[0].workspaces[0].windows.push(window(B, "b"));

    // Act
    let events = changes(&before, &after);

    // Assert
    assert_eq!(
        events,
        vec![
            DesktopEvent::WindowMoved {
                window: wid(B),
                monitor: mid(1),
                workspace: 0,
            },
            DesktopEvent::MonitorRemoved { monitor: mid(2) },
        ]
    );
}

#[test]
fn focus_on_another_monitor_reports_the_monitor() {
    // Arrange
    let mut before = desktop();
    before.monitors.push(monitor(2));
    let mut after = before.clone();
    after.focused_monitor = 1;
    after.focused_window = None;

    // Act
    let events = changes(&before, &after);

    // Assert
    assert_eq!(
        events,
        vec![DesktopEvent::FocusChanged {
            window: None,
            monitor: mid(2),
        }]
    );
}
