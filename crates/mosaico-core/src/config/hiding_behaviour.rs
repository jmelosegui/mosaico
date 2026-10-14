use serde::{Deserialize, Serialize};

/// How windows are hidden when switching away from their workspace.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HidingBehaviour {
    /// DWM Cloak: window becomes invisible but keeps its taskbar icon
    /// and does not fire `EVENT_OBJECT_HIDE`. Recommended default.
    #[default]
    Cloak,
    /// `ShowWindow(SW_HIDE)`: window is fully hidden and loses its
    /// taskbar icon. Fires `EVENT_OBJECT_HIDE`.
    Hide,
    /// `ShowWindow(SW_MINIMIZE)`: window is minimized. Keeps taskbar
    /// icon but shows minimized state. Fires `EVENT_SYSTEM_MINIMIZESTART`.
    Minimize,
}
