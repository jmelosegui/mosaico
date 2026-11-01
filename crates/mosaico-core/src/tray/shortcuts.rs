//! What the icon opens: the settings app, the config folder and the log.

use std::path::PathBuf;

use crate::system::{FileProbe, Opener, ProcessControl};

use super::TrayNotice;

/// Where the things the icon opens are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayPaths {
    /// The settings app, installed beside mosaico.
    pub settings: PathBuf,
    /// The folder with the config files.
    pub config_dir: PathBuf,
    /// The log file, which exists once logging is on.
    pub log: PathBuf,
}

/// Opens what the icon's menu names, through the OS services that do it.
/// Each method returns the notice to show when it could not.
pub struct Shortcuts<'a> {
    /// Where everything is.
    pub paths: &'a TrayPaths,
    /// Tells whether a file is there.
    pub files: &'a dyn FileProbe,
    /// Starts the settings app.
    pub processes: &'a dyn ProcessControl,
    /// Opens folders and files.
    pub opener: &'a dyn Opener,
}

impl Shortcuts<'_> {
    /// Starts the settings app, which brings forward a window already
    /// open.
    ///
    /// # Errors
    ///
    /// The notice to show when the app is not installed or cannot start.
    pub fn open_settings(&self) -> Result<(), TrayNotice> {
        let settings = &self.paths.settings;
        if self.files.is_file(settings) == Some(false) {
            return Err(TrayNotice::new(
                "The settings app is not installed",
                "Run 'mosaico update --force' to install it.",
            ));
        }
        self.processes
            .launch(settings)
            .map(drop)
            .map_err(|e| TrayNotice::new("Could not open the settings", e.to_string()))
    }

    /// Opens the config folder in the file manager.
    ///
    /// # Errors
    ///
    /// The notice to show when it cannot be opened.
    pub fn open_config_folder(&self) -> Result<(), TrayNotice> {
        self.opener
            .open(&self.paths.config_dir)
            .map_err(|e| TrayNotice::new("Could not open the config folder", e.to_string()))
    }

    /// Opens the log file.
    ///
    /// # Errors
    ///
    /// The notice to show when there is no log, because logging is off,
    /// or it cannot be opened.
    pub fn open_log(&self) -> Result<(), TrayNotice> {
        if self.files.is_file(&self.paths.log) == Some(false) {
            return Err(TrayNotice::new(
                "There is no log yet",
                "Set enabled = true under [logging] in config.toml to write one.",
            ));
        }
        self.opener
            .open(&self.paths.log)
            .map_err(|e| TrayNotice::new("Could not open the log", e.to_string()))
    }
}

#[cfg(test)]
#[path = "tests/shortcuts_tests.rs"]
mod tests;
