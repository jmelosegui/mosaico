//! Telling whether what is registered to start at login still starts
//! this mosaico.

use std::path::Path;

use super::{FileProbe, LaunchCommand};

/// How the registration compares with what this mosaico registers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutostartStatus {
    /// Nothing is registered.
    Disabled,
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
