use crate::api::{Daemon, Focus, FocusedWindow, Monitor, State, Workspace};
use crate::id::wid;
use crate::{LayoutKind, StableMonitorId};

use super::BarState;

fn workspace(number: u32, layout: &str) -> Workspace {
    Workspace {
        number,
        name: number.to_string(),
        layout: layout.into(),
        ..Workspace::default()
    }
}

/// Two monitors, `m-a` with three workspaces, the second on screen, and
/// `m-b`, with focus on window 7 of `m-a`.
fn state() -> State {
    let monitor = |id: &str, index, workspaces| Monitor {
        id: StableMonitorId::new(id),
        index,
        active_workspace: 2,
        workspaces,
        ..Monitor::default()
    };
    let a = vec![
        workspace(1, "bsp"),
        workspace(2, "three-column"),
        workspace(3, "bsp"),
    ];
    let b = vec![workspace(1, "bsp"), workspace(2, "spiral")];
    State {
        daemon: Daemon {
            update: Some("v1.0.0 available".into()),
            paused: true,
            wrap: true,
            ..Daemon::default()
        },
        monitors: vec![monitor("m-a", 0, a), monitor("m-b", 1, b)],
        focus: Focus {
            monitor: Some(StableMonitorId::new("m-a")),
            workspace: Some(2),
            window: Some(FocusedWindow {
                id: wid(7),
                title: "notes".into(),
                program: "notes".into(),
            }),
        },
        ..State::default()
    }
}

#[test]
fn the_focused_monitors_bar_shows_its_workspace_and_window() {
    // Arrange
    let state = state();

    // Act
    let bar = BarState::from_state(&state, &StableMonitorId::new("m-a"));

    // Assert
    let expected = BarState {
        active_workspace: 1,
        workspace_count: 3,
        layout: LayoutKind::ThreeColumn,
        monocle: false,
        focused_window: Some(wid(7)),
        update_text: "v1.0.0 available".into(),
        paused: true,
        wrap: true,
        maximized: false,
    };
    assert_eq!(bar, Some(expected));
}

#[test]
fn another_monitors_bar_shows_no_window_and_an_unknown_layout_as_the_default() {
    // Arrange
    let state = state();

    // Act
    let bar = BarState::from_state(&state, &StableMonitorId::new("m-b"));

    // Assert
    let bar = bar.unwrap_or_default();
    assert_eq!(bar.focused_window, None);
    assert_eq!(bar.layout, LayoutKind::default());
}

#[test]
fn a_monitor_the_state_does_not_have_has_no_bar_state() {
    // Arrange
    let state = state();

    // Act
    let bar = BarState::from_state(&state, &StableMonitorId::new("m-z"));

    // Assert
    assert_eq!(bar, None);
}
