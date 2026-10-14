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
///   else. Once it succeeds, `registered_command` returns an equal
///   command. A command the platform cannot store so that it reads back
///   the same is refused with
///   [`Error::Unsupported`](crate::Error::Unsupported), and the
///   registration is left as it was.
/// - `registered_command` reports the program and arguments that run at
///   login as they were registered: it does not resolve links, and it
///   keeps an entry whose program is gone. Where a registration is a
///   plain command line, the program in it is found the way the system
///   finds it at login, without waiting on the network. Nothing follows
///   the executable: after it moves or is deleted, the old path is
///   reported until `enable` runs again.
/// - `registered_command` returns `None` only when nothing is
///   registered. An entry that names no program comes back with an
///   empty `program`, and one that cannot be read is an error.
/// - `disable` succeeds when nothing was registered.
/// - A registration, removal or read that fails in the system is an
///   [`Error::Platform`](crate::Error::Platform).
/// - A platform without a way to start at login returns
///   [`Error::Unsupported`](crate::Error::Unsupported) from `enable`,
///   `None` from `registered_command`, and succeeds in `disable`.
pub trait Autostart {
    /// What runs when the user logs in, or `None` when nothing is
    /// registered.
    fn registered_command(&self) -> Result<Option<LaunchCommand>>;

    /// Registers `command` to run when the user logs in.
    fn enable(&self, command: &LaunchCommand) -> Result<()>;

    /// Removes the registration.
    fn disable(&self) -> Result<()>;
}

#[cfg(test)]
#[path = "tests/autostart_tests.rs"]
mod tests;
