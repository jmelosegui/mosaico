//! The names of the desktop events, which clients use to pick the ones
//! they want.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A kind of [`DesktopEvent`](super::DesktopEvent), named as the event's
/// `event` field names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// `window_managed`
    WindowManaged,
    /// `window_unmanaged`
    WindowUnmanaged,
    /// `window_moved`
    WindowMoved,
    /// `window_title_changed`
    WindowTitleChanged,
    /// `window_floating_changed`
    WindowFloatingChanged,
    /// `focus_changed`
    FocusChanged,
    /// `workspace_changed`
    WorkspaceChanged,
    /// `layout_changed`
    LayoutChanged,
    /// `monitor_added`
    MonitorAdded,
    /// `monitor_removed`
    MonitorRemoved,
    /// `paused`
    Paused,
    /// `config_reloaded`
    ConfigReloaded,
}

impl EventKind {
    /// Every kind, in the order the documentation lists them.
    pub const ALL: [Self; 12] = [
        Self::WindowManaged,
        Self::WindowUnmanaged,
        Self::WindowMoved,
        Self::WindowTitleChanged,
        Self::WindowFloatingChanged,
        Self::FocusChanged,
        Self::WorkspaceChanged,
        Self::LayoutChanged,
        Self::MonitorAdded,
        Self::MonitorRemoved,
        Self::Paused,
        Self::ConfigReloaded,
    ];

    /// The name clients know the kind by.
    pub fn name(self) -> &'static str {
        match self {
            Self::WindowManaged => "window_managed",
            Self::WindowUnmanaged => "window_unmanaged",
            Self::WindowMoved => "window_moved",
            Self::WindowTitleChanged => "window_title_changed",
            Self::WindowFloatingChanged => "window_floating_changed",
            Self::FocusChanged => "focus_changed",
            Self::WorkspaceChanged => "workspace_changed",
            Self::LayoutChanged => "layout_changed",
            Self::MonitorAdded => "monitor_added",
            Self::MonitorRemoved => "monitor_removed",
            Self::Paused => "paused",
            Self::ConfigReloaded => "config_reloaded",
        }
    }
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A name that is not an [`EventKind`], so a client that misspells one
/// hears about it instead of waiting for an event that never comes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownEvent(pub String);

impl fmt::Display for UnknownEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown event `{}`; known events are ", self.0)?;
        for (i, kind) in EventKind::ALL.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            f.write_str(kind.name())?;
        }
        Ok(())
    }
}

impl std::error::Error for UnknownEvent {}

impl FromStr for EventKind {
    type Err = UnknownEvent;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.name() == s)
            .ok_or_else(|| UnknownEvent(s.to_string()))
    }
}
