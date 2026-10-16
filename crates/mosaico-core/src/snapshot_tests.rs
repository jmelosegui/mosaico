use super::*;
use crate::ipc::Response;

fn key(name: &str) -> MonitorKey {
    MonitorKey {
        stable_id: Some(name.into()),
        ..Default::default()
    }
}

fn window(raw: u64, title: &str) -> WindowSnapshot {
    WindowSnapshot {
        id: WindowId::from_raw(raw),
        visible: true,
        minimized: false,
        cloaked: false,
        elevated: false,
        rect: Some(Rect::new(0, 40, 960, 1040)),
        class: "Notepad".into(),
        title: title.into(),
    }
}

/// Two monitors: the first on its second workspace with three windows
/// over two workspaces, the second with none.
fn snapshot() -> DesktopSnapshot {
    let hidden = WindowSnapshot {
        cloaked: true,
        rect: None,
        ..window(0x5020A, "b \"quoted\"")
    };
    DesktopSnapshot {
        monitors: vec![
            MonitorSnapshot {
                id: MonitorId::from_raw(0x10001),
                key: key("DISPLAY1"),
                work_area: Rect::new(0, 40, 1920, 1040),
                active_workspace: 1,
                workspaces: vec![
                    WorkspaceSnapshot::default(),
                    WorkspaceSnapshot {
                        windows: vec![window(0x4014E, "a.txt - Notepad"), hidden],
                    },
                    WorkspaceSnapshot {
                        windows: vec![window(0x6030C, "c")],
                    },
                ],
            },
            MonitorSnapshot {
                id: MonitorId::from_raw(0x20003),
                key: MonitorKey::default(),
                work_area: Rect::new(1920, 0, 1920, 1080),
                active_workspace: 0,
                workspaces: vec![WorkspaceSnapshot::default(); 8],
            },
        ],
        disconnected: vec![key("DISPLAY2"), key("DISPLAY3")],
        displaced_windows: 2,
        daemon_elevated: false,
        focused_monitor: 0,
        focused_window: Some(WindowId::from_raw(0x4014E)),
    }
}

#[test]
fn the_snapshot_prints_as_the_debug_state_text() {
    // Act
    let text = snapshot().to_string();

    // Assert -- the text `debug state` printed before the payload was
    // typed, which scripts and the display tests read.
    let expected = r#"Monitor 0 (id=0x10001, key=[DISPLAY1], work_area=1920x1040+0+40, active_ws=2):
  Workspace 2 (2 windows):
    0x4014E  visible=true  minimized=false  cloaked=false  elevated=false  rect=960x1040+0+40  class="Notepad"  title="a.txt - Notepad"
    0x5020A  visible=true  minimized=false  cloaked=true  elevated=false  rect=N/A  class="Notepad"  title="b \"quoted\""
  Workspace 3 (1 window):
    0x6030C  visible=true  minimized=false  cloaked=false  elevated=false  rect=960x1040+0+40  class="Notepad"  title="c"
Monitor 1 (id=0x20003, key=unknown, work_area=1920x1080+1920+0, active_ws=1):
Disconnected monitors: [DISPLAY2]; [DISPLAY3], displaced windows: 2
Self elevated: false
Focused monitor: 0
Focused window: 0x4014E
"#;
    assert_eq!(text, expected);
}

#[test]
fn nothing_disconnected_or_focused_prints_as_none() {
    // Arrange
    let quiet = DesktopSnapshot {
        monitors: Vec::new(),
        disconnected: Vec::new(),
        displaced_windows: 0,
        daemon_elevated: true,
        focused_monitor: 0,
        focused_window: None,
    };

    // Act
    let text = quiet.to_string();

    // Assert
    assert_eq!(
        text,
        "Disconnected monitors: none, displaced windows: 0\n\
         Self elevated: true\n\
         Focused monitor: 0\n\
         Focused window: None\n"
    );
}

#[test]
fn the_snapshot_travels_as_data() {
    // Arrange
    let sent = snapshot();

    // Act
    let response = Response::ok().with_data(&sent);

    // Assert
    assert_eq!(response.data::<DesktopSnapshot>(), Some(sent));
}
