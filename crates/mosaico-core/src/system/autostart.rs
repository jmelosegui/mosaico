//! Starting mosaico when the user logs in.

use std::fmt;
use std::path::PathBuf;

use crate::Result;

/// A program and the arguments it is started with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchCommand {
    /// The executable to run.
    pub program: PathBuf,
    /// The arguments it gets, in order.
    pub args: Vec<String>,
}

impl LaunchCommand {
    /// What mosaico registers to start at login: `exe start`, where
    /// `exe` is the mosaico executable to run.
    ///
    /// Enabling autostart registers this with the executable that is
    /// running, so after the binary moves, enabling again points the
    /// registration at its new place.
    pub fn autostart(exe: impl Into<PathBuf>) -> Self {
        Self {
            program: exe.into(),
            args: vec!["start".to_string()],
        }
    }
}

/// Shows the program in quotes, then the arguments, quoting those with
/// whitespace, for telling the user what runs.
impl fmt::Display for LaunchCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.program.display())?;
        for arg in &self.args {
            if arg.is_empty() || arg.contains(char::is_whitespace) {
                write!(f, " \"{arg}\"")?;
            } else {
                write!(f, " {arg}")?;
            }
        }
        Ok(())
    }
}

/// Starting mosaico when the user logs in.
///
/// # Contract for platform implementations
///
/// - Registration is per user and needs no elevation. A user has one
///   registration: `enable` replaces whatever is registered, including
///   an entry another copy of mosaico made, and `disable` removes it.
/// - `enable` registers exactly the command it is given: at login the
///   platform runs its `program` with its `args`, in order, and nothing
///   else.
/// - `disable` succeeds when nothing was registered.
/// - A platform without a way to start at login returns
///   [`Error::Unsupported`](crate::Error::Unsupported) from `enable`,
///   and `is_enabled` returns false.
pub trait Autostart {
    /// Whether mosaico is registered to start at login.
    fn is_enabled(&self) -> bool;

    /// Registers `command` to run when the user logs in.
    fn enable(&self, command: &LaunchCommand) -> Result<()>;

    /// Removes the registration.
    fn disable(&self) -> Result<()>;
}

#[cfg(test)]
#[path = "tests/autostart_tests.rs"]
mod tests;
