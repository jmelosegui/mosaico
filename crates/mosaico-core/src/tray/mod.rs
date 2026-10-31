//! The mosaico icon in the notification area: what its menu offers, what
//! its tooltip says, and the contract for putting it on screen.

mod badge;
mod controller;
mod shortcuts;
mod source;

pub use badge::{PAUSED_BADGE, paint_paused};
pub use controller::Tray;
pub use shortcuts::{Shortcuts, TrayPaths};
pub use source::{TrayControl, TraySource, TrayWatch};

/// What the user asks for through the icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    /// Opens the settings app, or brings its window forward.
    OpenSettings,
    /// Pauses the hotkeys, or resumes them, as `toggle-pause` does.
    TogglePause,
    /// Loads every config file again, as if each had changed.
    ReloadConfig,
    /// Opens the config folder in the file manager.
    OpenConfigFolder,
    /// Opens the log file.
    OpenLog,
    /// Stops the daemon, as the `stop` command does.
    Quit,
}

impl TrayCommand {
    /// What a click on the icon itself does. The menu shows it first and
    /// marks it as the default.
    pub const DEFAULT: Self = Self::OpenSettings;
}

/// An entry in the icon's menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayItem {
    /// An entry that sends `command` when picked, shown as `label`.
    Command {
        /// What picking the entry sends.
        command: TrayCommand,
        /// What the entry says.
        label: String,
    },
    /// A line between groups of entries.
    Separator,
}

/// What the icon shows: the version for the tooltip, and whether the
/// hotkeys are paused, which picks the icon, the tooltip and the label of
/// the pause entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayState {
    /// The version of the running mosaico.
    pub version: String,
    /// Whether the hotkeys are paused.
    pub paused: bool,
}

impl TrayState {
    /// The state of a daemon of `version` that has just started.
    pub fn new(version: &str) -> Self {
        Self {
            version: version.to_string(),
            paused: false,
        }
    }

    /// The text shown when the pointer rests on the icon.
    pub fn tooltip(&self) -> String {
        let version = &self.version;
        if self.paused {
            format!("mosaico {version} (hotkeys paused)")
        } else {
            format!("mosaico {version}")
        }
    }

    /// The entries of the icon's menu, top to bottom.
    pub fn menu(&self) -> Vec<TrayItem> {
        let pause = if self.paused {
            "Resume hotkeys"
        } else {
            "Pause hotkeys"
        };
        vec![
            item(TrayCommand::OpenSettings, "Settings\u{2026}"),
            TrayItem::Separator,
            item(TrayCommand::TogglePause, pause),
            item(TrayCommand::ReloadConfig, "Reload config"),
            TrayItem::Separator,
            item(TrayCommand::OpenConfigFolder, "Open config folder"),
            item(TrayCommand::OpenLog, "Open log"),
            TrayItem::Separator,
            item(TrayCommand::Quit, "Quit mosaico"),
        ]
    }
}

fn item(command: TrayCommand, label: &str) -> TrayItem {
    TrayItem::Command {
        command,
        label: label.to_string(),
    }
}

/// A short message shown beside the icon, that goes away by itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayNotice {
    /// The first line, in bold.
    pub title: String,
    /// The text under the title.
    pub body: String,
}

impl TrayNotice {
    /// A notice with `title` and `body`.
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
        }
    }

    /// Tells that `file` could not be loaded because of `reason`, and that
    /// the settings in use stay.
    ///
    /// A notification has little room, so a reason of several lines, such
    /// as a parse error quoting the file, is cut to its first line, where
    /// the problem is, and its last, what it is.
    pub fn invalid_config(file: &str, reason: &str) -> Self {
        let mut lines = reason
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty());
        let first = lines.next().unwrap_or_default();
        let reason = match lines.next_back() {
            Some(last) => format!("{first}: {last}"),
            None => first.to_owned(),
        };
        Self::new(
            format!("{file} was not loaded"),
            format!("{reason}\nThe settings in use stay until the file is fixed."),
        )
    }

    /// Tells that every config file was loaded again.
    pub fn reloaded() -> Self {
        Self::new(
            "Config reloaded",
            "Changes to keybindings take effect when mosaico restarts.",
        )
    }
}

#[cfg(test)]
#[path = "tests/tray_tests.rs"]
mod tests;
