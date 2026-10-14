//! A desktop for testing commands that list windows and monitors.

use mosaico_core::tiler::{Display, Enumerator, Insets, WindowDetails, WindowFacts};
use mosaico_core::{Error, MonitorId, MonitorKey, Rect, Result, WindowId};

/// Windows and monitors, as a platform reports them.
#[derive(Default)]
pub struct FakeDesktop {
    /// Windows front to back, with what the platform tells about each.
    pub windows: Vec<(WindowId, WindowDetails)>,
    /// Monitors, in the order the platform finds them.
    pub monitors: Vec<Display>,
    /// Whether the system refuses to list the windows.
    pub windows_unreadable: bool,
    /// Whether the system refuses to list the monitors.
    pub monitors_unreadable: bool,
}

impl Enumerator for FakeDesktop {
    fn windows(&self) -> Result<Vec<WindowId>> {
        if self.windows_unreadable {
            return Err(Error::platform("list windows", "access denied"));
        }
        Ok(self.windows.iter().map(|(id, _)| *id).collect())
    }

    fn connected_monitors(&self) -> Result<Vec<Display>> {
        if self.monitors_unreadable {
            return Err(Error::platform("list monitors", "access denied"));
        }
        Ok(self.monitors.clone())
    }

    fn details(&self, window: WindowId) -> WindowDetails {
        self.windows
            .iter()
            .find(|(id, _)| *id == window)
            .map(|(_, details)| details.clone())
            .unwrap_or_else(|| WindowDetails {
                facts: WindowFacts::default(),
                rect: None,
                minimized: false,
                maximized: false,
                monitor: MonitorId::from_raw(0),
                frame: None,
            })
    }
}

/// Monitor `id`, a 1920x1040 work area whose left edge is at `x`. The
/// one at the origin is primary.
pub fn monitor(id: u64, x: i32) -> Display {
    Display {
        id: MonitorId::from_raw(id),
        key: MonitorKey {
            stable_id: Some(format!("mon{id}")),
            ..Default::default()
        },
        work_area: Rect::new(x, 0, 1920, 1040),
        primary: x == 0,
    }
}

/// A normal application window with a title bar, on monitor `monitor`.
pub fn app_window(monitor: u64) -> WindowDetails {
    WindowDetails {
        facts: WindowFacts {
            class: "App".into(),
            title: "app".into(),
            path: "C:\\apps\\app.exe".into(),
            has_caption: true,
        },
        rect: Some(Rect::new(100, 100, 640, 480)),
        minimized: false,
        maximized: false,
        monitor: MonitorId::from_raw(monitor),
        frame: Some(Insets {
            left: 7,
            top: 0,
            right: 7,
            bottom: 7,
        }),
    }
}
