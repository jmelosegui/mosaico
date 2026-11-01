//! The `[tray]` section of `config.toml`: the icon in the notification
//! area.

use serde::{Deserialize, Serialize};

/// Notification area settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TrayConfig {
    /// Show the mosaico icon in the notification area while the daemon
    /// runs. Its menu opens the settings, pauses the hotkeys, reloads the
    /// config and quits.
    pub enabled: bool,
}

impl Default for TrayConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}
