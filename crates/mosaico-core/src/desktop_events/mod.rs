//! What changed on the desktop, as a client that subscribes sees it.
//!
//! The daemon compares the desktop before and after each step with
//! [`changes`] and sends what it finds to its subscribers, so every
//! event is decided here, in one place, and the tiler never knows who
//! listens. Events only report what happened: a subscriber cannot stop
//! or change what the daemon does.

mod changes;
mod kind;

use serde::{Deserialize, Serialize};

use crate::layout::LayoutKind;
use crate::{MonitorId, WindowId};

pub use changes::changes;
pub use kind::{EventKind, UnknownEvent};

/// Something that changed on the desktop.
///
/// Travels as one JSON object whose `event` field names it with its
/// [`EventKind`], such as `{"event":"focus_changed",...}`. Clients must
/// ignore fields and events they do not know: later versions add them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum DesktopEvent {
    /// The daemon started managing a window.
    WindowManaged {
        /// The window.
        window: WindowId,
        /// The monitor it is on.
        monitor: MonitorId,
        /// Index of the workspace it is on.
        workspace: usize,
        /// Window class or the platform's nearest equivalent.
        class: String,
        /// Window title.
        title: String,
    },
    /// The daemon stopped managing a window, usually because it closed.
    WindowUnmanaged {
        /// The window.
        window: WindowId,
    },
    /// A window moved to another workspace or monitor.
    WindowMoved {
        /// The window.
        window: WindowId,
        /// The monitor it is on now.
        monitor: MonitorId,
        /// Index of the workspace it is on now.
        workspace: usize,
    },
    /// A window's title changed.
    WindowTitleChanged {
        /// The window.
        window: WindowId,
        /// The new title.
        title: String,
    },
    /// A window started or stopped floating.
    WindowFloatingChanged {
        /// The window.
        window: WindowId,
        /// Whether it floats now.
        floating: bool,
    },
    /// Focus moved to another window or monitor.
    FocusChanged {
        /// The window with focus, if any.
        window: Option<WindowId>,
        /// The monitor the user is working on.
        monitor: MonitorId,
    },
    /// A monitor shows another workspace.
    WorkspaceChanged {
        /// The monitor.
        monitor: MonitorId,
        /// Index of the workspace it shows now.
        workspace: usize,
    },
    /// A workspace changed its layout or turned monocle on or off.
    LayoutChanged {
        /// The monitor the workspace belongs to.
        monitor: MonitorId,
        /// Index of the workspace.
        workspace: usize,
        /// The layout that tiles it now.
        layout: LayoutKind,
        /// Whether one window fills it now.
        monocle: bool,
    },
    /// A monitor was connected.
    MonitorAdded {
        /// The monitor.
        monitor: MonitorId,
    },
    /// A monitor was disconnected.
    MonitorRemoved {
        /// The monitor.
        monitor: MonitorId,
    },
    /// The hotkeys were paused or resumed.
    Paused {
        /// Whether they are paused now.
        paused: bool,
    },
    /// The daemon loaded a changed configuration file.
    ConfigReloaded,
}

impl DesktopEvent {
    /// Which kind of event this is.
    pub fn kind(&self) -> EventKind {
        match self {
            Self::WindowManaged { .. } => EventKind::WindowManaged,
            Self::WindowUnmanaged { .. } => EventKind::WindowUnmanaged,
            Self::WindowMoved { .. } => EventKind::WindowMoved,
            Self::WindowTitleChanged { .. } => EventKind::WindowTitleChanged,
            Self::WindowFloatingChanged { .. } => EventKind::WindowFloatingChanged,
            Self::FocusChanged { .. } => EventKind::FocusChanged,
            Self::WorkspaceChanged { .. } => EventKind::WorkspaceChanged,
            Self::LayoutChanged { .. } => EventKind::LayoutChanged,
            Self::MonitorAdded { .. } => EventKind::MonitorAdded,
            Self::MonitorRemoved { .. } => EventKind::MonitorRemoved,
            Self::Paused { .. } => EventKind::Paused,
            Self::ConfigReloaded => EventKind::ConfigReloaded,
        }
    }
}

#[cfg(test)]
mod tests;
