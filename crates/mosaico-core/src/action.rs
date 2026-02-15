use serde::{Deserialize, Serialize};

/// An action that can be triggered by a hotkey or CLI command.
///
/// Actions represent user-initiated operations on the tiling manager.
/// They are distinct from window events (which come from the OS).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    /// Move focus to the next window in the workspace.
    FocusNext,
    /// Move focus to the previous window in the workspace.
    FocusPrev,
    /// Swap the focused window with the next one.
    SwapNext,
    /// Swap the focused window with the previous one.
    SwapPrev,
    /// Re-apply the current layout to all managed windows.
    Retile,
    /// Move focus to a window on the next monitor.
    FocusMonitorNext,
    /// Move focus to a window on the previous monitor.
    FocusMonitorPrev,
    /// Move the focused window to the next monitor.
    MoveToMonitorNext,
    /// Move the focused window to the previous monitor.
    MoveToMonitorPrev,
}
