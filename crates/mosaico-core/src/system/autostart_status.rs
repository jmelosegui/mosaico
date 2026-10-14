//! Telling whether what is registered to start at login still starts
//! this mosaico.

use std::path::{Path, PathBuf};

use crate::Result;

use super::{Autostart, FileProbe, LaunchCommand};

/// How the registration compares with what this mosaico registers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutostartStatus {
    /// Nothing is registered.
    Disabled,
    /// An entry is registered, but the user turned it off in the system's
    /// startup settings, so it does not start at login.
    TurnedOff,
    /// An entry is registered, but it names no program, so nothing starts
    /// at login.
    Empty,
    /// This executable is registered, started the way enabling does it.
    Enabled,
    /// Another command is registered: another program, or this one with
    /// other arguments.
    Other {
        /// What is registered.
        registered: LaunchCommand,
        /// What enabling autostart from this executable registers.
        expected: LaunchCommand,
    },
    /// The registered program is not a file: nothing is at its path, or
    /// something else such as a directory, as after the binary was moved
    /// or deleted. Nothing starts at login.
    Missing(LaunchCommand),
}

impl AutostartStatus {
    /// Asks `autostart` what is registered and, unless the user turned it
    /// off, compares it with what enabling autostart from the running
    /// executable registers, see [`AutostartStatus::of`].
    ///
    /// `exe` finds the running executable. It is only asked when an entry
    /// is registered and turned on, so "disabled" and "turned off" are
    /// reported even when it cannot be found.
    pub fn read(
        autostart: &dyn Autostart,
        files: &dyn FileProbe,
        exe: impl FnOnce() -> Result<PathBuf>,
    ) -> Result<Self> {
        let Some(registered) = autostart.registered_command()? else {
            return Ok(Self::Disabled);
        };
        if autostart.is_turned_off()? {
            return Ok(Self::TurnedOff);
        }
        Ok(Self::of(Some(registered), &exe()?, files))
    }

    /// Compares `registered` with what enabling autostart from `exe`, the
    /// running executable, registers, asking `files` about the program.
    ///
    /// The registered program is this executable when the paths are equal
    /// or name the same file, so another spelling of its path, or a
    /// symbolic link or junction to it, still counts as this one. A
    /// relative program is another command, and `files` is not asked
    /// about it, since the file it names depends on where it is run from.
    /// A program `files` cannot check without waiting is another command
    /// too, rather than one that is missing.
    pub fn of(registered: Option<LaunchCommand>, exe: &Path, files: &dyn FileProbe) -> Self {
        let Some(registered) = registered else {
            return Self::Disabled;
        };
        if registered.program.as_os_str().is_empty() {
            return Self::Empty;
        }
        let expected = LaunchCommand::autostart(exe);
        let program = registered.program.as_path();
        if program.is_absolute() {
            let is_this =
                program == expected.program || files.same_file(program, &expected.program);
            if is_this && registered.args == expected.args {
                return Self::Enabled;
            }
            if !is_this && files.is_file(program) == Some(false) {
                return Self::Missing(registered);
            }
        }
        Self::Other {
            registered,
            expected,
        }
    }
}

#[cfg(test)]
#[path = "tests/autostart_status_tests.rs"]
mod tests;
