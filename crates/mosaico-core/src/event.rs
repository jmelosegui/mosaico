use std::fmt;

use crate::WindowId;

mod hover;
mod rest;

pub use hover::{FocusFollowsMouse, MousePointer, POLL_INTERVAL, REST_POLLS};

/// A platform-agnostic window event.
///
/// These represent meaningful state changes that the tiling manager
/// needs to react to. Platform crates translate raw OS events into
/// these variants.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    /// A new window was created and made visible.
    Created {
        /// The window.
        window: WindowId,
    },

    /// A window was destroyed or closed.
    Destroyed {
        /// The window.
        window: WindowId,
    },

    /// A window was hidden, but not necessarily destroyed.
    ///
    /// GPU-accelerated apps (e.g. Zed, browsers) may fire HIDE when
    /// recreating their rendering surface. The handler should verify the
    /// window is truly gone before removing it from the layout.
    Hidden {
        /// The window.
        window: WindowId,
    },

    /// A window received keyboard focus.
    Focused {
        /// The window.
        window: WindowId,
    },

    /// A window was focused via mouse hover (focus-follows-mouse).
    MouseHover {
        /// The window.
        window: WindowId,
    },

    /// A window finished being moved or resized.
    Moved {
        /// The window.
        window: WindowId,
    },

    /// A window was minimized.
    Minimized {
        /// The window.
        window: WindowId,
    },

    /// A window was restored from minimized state.
    Restored {
        /// The window.
        window: WindowId,
    },

    /// A window's title changed.
    TitleChanged {
        /// The window.
        window: WindowId,
    },

    /// A window's position or size changed.
    ///
    /// Fires frequently (e.g. during animations), so handlers should
    /// filter to the focused window and avoid expensive work.
    LocationChanged {
        /// The window.
        window: WindowId,
    },

    /// The display configuration changed (monitor connect/disconnect).
    DisplayChanged,

    /// The desktop work area changed (e.g. taskbar shown/hidden).
    WorkAreaChanged,
}

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
mod tests {
    use super::*;

    #[test]
    fn window_events_carry_their_window() {
        // Arrange
        let window = WindowId::from_raw(0x10);
        let event = WindowEvent::Focused { window };

        // Act / Assert
        assert_eq!(event.window(), Some(window));
        assert_eq!(event.to_string(), "Focused 0x10");
    }

    #[test]
    fn display_events_have_no_window() {
        // Arrange
        let event = WindowEvent::DisplayChanged;

        // Act / Assert
        assert_eq!(event.window(), None);
        assert_eq!(event.to_string(), "DisplayChanged");
    }
}
