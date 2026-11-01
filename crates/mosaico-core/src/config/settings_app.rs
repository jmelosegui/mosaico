//! `settings-app.toml`: what the settings window remembers between runs.
//! The daemon never reads it, and changing it reloads nothing.

use serde::{Deserialize, Serialize};

use super::ConfigStore;
use crate::Result;

/// The window's size when nothing is remembered, in points.
pub const SETTINGS_WINDOW_SIZE: (f32, f32) = (980.0, 700.0);

/// The least the window can be, in points.
pub const SETTINGS_WINDOW_MIN: (f32, f32) = (720.0, 480.0);

/// The most a remembered size can be, in points, so a broken file cannot
/// open a window larger than any screen.
const SETTINGS_WINDOW_MAX: f32 = 16_384.0;

/// What the settings window remembers between runs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SettingsAppConfig {
    /// The page shown when the window closed, by its name in the
    /// navigation. Empty, or a name no page has, opens the first page.
    pub page: String,
    /// The window's width, in points.
    pub width: f32,
    /// The window's height, in points.
    pub height: f32,
    /// Whether the window filled the screen.
    pub maximized: bool,
}

impl Default for SettingsAppConfig {
    fn default() -> Self {
        Self {
            page: String::new(),
            width: SETTINGS_WINDOW_SIZE.0,
            height: SETTINGS_WINDOW_SIZE.1,
            maximized: false,
        }
    }
}

impl SettingsAppConfig {
    /// The size to open the window at: the one remembered, brought
    /// between the least and the most a window can be.
    pub fn size(&self) -> (f32, f32) {
        let fit = |value: f32, min: f32, default: f32| {
            if value.is_finite() {
                value.clamp(min, SETTINGS_WINDOW_MAX)
            } else {
                default
            }
        };
        (
            fit(self.width, SETTINGS_WINDOW_MIN.0, SETTINGS_WINDOW_SIZE.0),
            fit(self.height, SETTINGS_WINDOW_MIN.1, SETTINGS_WINDOW_SIZE.1),
        )
    }
}

impl ConfigStore {
    /// `settings-app.toml`: what the settings window remembers.
    pub fn settings_app_path(&self) -> std::path::PathBuf {
        self.dir().join("settings-app.toml")
    }

    /// What the settings window remembers, or the defaults when the file
    /// is missing or does not parse. Only the window writes the file, so
    /// a broken one is not reported: it is replaced on the next save.
    pub fn load_settings_app(&self) -> SettingsAppConfig {
        std::fs::read_to_string(self.settings_app_path())
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    }

    /// Writes what the settings window remembers. Does nothing when the
    /// file already says it, so closing an unchanged window writes
    /// nothing. Creates the config directory when it does not exist.
    ///
    /// # Errors
    ///
    /// When the file cannot be written.
    pub fn save_settings_app(&self, app: &SettingsAppConfig) -> Result<()> {
        if self.settings_app_path().exists() && self.load_settings_app() == *app {
            return Ok(());
        }
        let path = self.settings_app_path();
        let text = toml::to_string(app).map_err(|e| super::store::file_error(&path, e))?;
        std::fs::create_dir_all(self.dir()).map_err(|e| super::store::file_error(&path, e))?;
        std::fs::write(&path, text).map_err(|e| super::store::file_error(&path, e))
    }
}

#[cfg(test)]
#[path = "settings_app_tests.rs"]
mod tests;
