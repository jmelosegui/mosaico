//! Which window an event is about, and how events are printed.

use std::fmt;

use crate::{WindowEvent, WindowId};

impl WindowEvent {
    /// The window the event is about, or `None` for display and work
    /// area events, which concern no window in particular.
    pub fn window(&self) -> Option<WindowId> {
        match *self {
            Self::Created { window }
            | Self::Destroyed { window }
            | Self::Hidden { window }
            | Self::Focused { window }
            | Self::MouseHover { window }
            | Self::Moved { window }
            | Self::Minimized { window }
            | Self::Restored { window }
            | Self::TitleChanged { window }
            | Self::LocationChanged { window } => Some(window),
            Self::DisplayChanged | Self::WorkAreaChanged => None,
        }
    }

    /// Returns the event name without the window.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "Created",
            Self::Destroyed { .. } => "Destroyed",
            Self::Hidden { .. } => "Hidden",
            Self::Focused { .. } => "Focused",
            Self::MouseHover { .. } => "MouseHover",
            Self::Moved { .. } => "Moved",
            Self::Minimized { .. } => "Minimized",
            Self::Restored { .. } => "Restored",
            Self::TitleChanged { .. } => "TitleChanged",
            Self::LocationChanged { .. } => "LocationChanged",
            Self::DisplayChanged => "DisplayChanged",
            Self::WorkAreaChanged => "WorkAreaChanged",
        }
    }
}

impl fmt::Display for WindowEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.window() {
            Some(window) => write!(f, "{} {}", self.name(), window),
            None => f.write_str(self.name()),
        }
    }
}

#[cfg(test)]
#[path = "tests/naming_tests.rs"]
mod tests;
