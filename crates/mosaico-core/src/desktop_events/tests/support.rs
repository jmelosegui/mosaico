pub(super) use super::super::*;
pub(super) use crate::layout::LayoutKind;
pub(super) use crate::monitor::MonitorKey;
pub(super) use crate::snapshot::{
    DesktopSnapshot, MonitorSnapshot, WindowSnapshot, WorkspaceSnapshot,
};
pub(super) use crate::{MonitorId, Rect, WindowId};

pub(super) const A: u64 = 0x100;
pub(super) const B: u64 = 0x200;

pub(super) fn mid(raw: u64) -> MonitorId {
    MonitorId::from_raw(raw)
}

pub(super) fn wid(raw: u64) -> WindowId {
    WindowId::from_raw(raw)
}

pub(super) fn window(raw: u64, title: &str) -> WindowSnapshot {
    WindowSnapshot {
        id: wid(raw),
        visible: true,
        minimized: false,
        cloaked: false,
        elevated: false,
        rect: None,
        class: "Notepad".into(),
        title: title.into(),
    }
}

pub(super) fn monitor(raw: u64) -> MonitorSnapshot {
    MonitorSnapshot {
        id: mid(raw),
        key: MonitorKey::default(),
        work_area: Rect::new(0, 0, 1920, 1080),
        active_workspace: 0,
        workspaces: vec![WorkspaceSnapshot::default(); 3],
    }
}

/// One monitor with window A on its first workspace, focused.
pub(super) fn desktop() -> DesktopSnapshot {
    let mut mon = monitor(1);
    mon.workspaces[0].windows.push(window(A, "a"));
    DesktopSnapshot {
        monitors: vec![mon],
        disconnected: Vec::new(),
        displaced_windows: 0,
        daemon_elevated: false,
        focused_monitor: 0,
        focused_window: Some(wid(A)),
    }
}
