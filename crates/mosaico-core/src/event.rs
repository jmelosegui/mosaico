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
        }
    }
}
