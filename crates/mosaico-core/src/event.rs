use std::fmt;

/// A platform-agnostic window event.
///
/// These represent meaningful state changes that the tiling manager
/// needs to react to. Platform crates translate raw OS events into
/// these variants.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    /// A new window was created and made visible.
    Created { hwnd: usize },

    /// A window was destroyed or closed.
    Destroyed { hwnd: usize },

    /// A window received keyboard focus.
    Focused { hwnd: usize },

    /// A window finished being moved or resized.
    Moved { hwnd: usize },

    /// A window was minimized.
    Minimized { hwnd: usize },

    /// A window was restored from minimized state.
    Restored { hwnd: usize },

    /// A window's title changed.
    TitleChanged { hwnd: usize },

    /// The display configuration changed (monitor connect/disconnect).
    DisplayChanged,
}

impl WindowEvent {
    /// Returns the window handle associated with this event.
    pub fn hwnd(&self) -> usize {
        match self {
            Self::Created { hwnd }
            | Self::Destroyed { hwnd }
            | Self::Focused { hwnd }
            | Self::Moved { hwnd }
            | Self::Minimized { hwnd }
            | Self::Restored { hwnd }
            | Self::TitleChanged { hwnd } => *hwnd,
            Self::DisplayChanged => 0,
        }
    }

    /// Returns the event name without the handle.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Created { .. } => "Created",
            Self::Destroyed { .. } => "Destroyed",
            Self::Focused { .. } => "Focused",
            Self::Moved { .. } => "Moved",
            Self::Minimized { .. } => "Minimized",
            Self::Restored { .. } => "Restored",
            Self::TitleChanged { .. } => "TitleChanged",
            Self::DisplayChanged => "DisplayChanged",
        }
    }
}

impl fmt::Display for WindowEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} 0x{:X}", self.name(), self.hwnd())
    }
}
