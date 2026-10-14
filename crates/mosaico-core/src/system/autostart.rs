//! Starting mosaico when the user logs in, and telling whether what is
//! registered still starts this mosaico.

use std::fmt;
use std::path::{Path, PathBuf};

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
/// - `registered_command` reports what runs at login as it was
///   registered, without resolving or checking the program. Nothing
///   follows the executable: after it moves or is deleted, the old path
///   is reported until `enable` runs again.
/// - `registered_command` returns `None` only when nothing is
///   registered. A registration that cannot be read, or that names no
///   program, is an error.
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

/// How the registration compares with what this mosaico registers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutostartStatus {
    /// Nothing is registered.
    Disabled,
    /// This executable is registered, started the way enabling does it.
    Enabled,
    /// Another program, or this one with other arguments, is registered.
    Other(LaunchCommand),
    /// The registered program does not exist, as after it was moved or
    /// deleted, so nothing starts at login.
    Missing(LaunchCommand),
}

impl AutostartStatus {
    /// Compares `registered` with what enabling autostart from `exe`, the
    /// running executable, registers.
    ///
    /// Paths name the same program when they are equal or resolve to the
    /// same file, so another spelling of the running executable's path,
    /// or a link to it, still counts as this one.
    pub fn of(registered: Option<LaunchCommand>, exe: &Path) -> Self {
        let Some(registered) = registered else {
            return Self::Disabled;
        };
        let expected = LaunchCommand::autostart(exe);
        if is_same_program(&registered.program, &expected.program) {
            if registered.args == expected.args {
                Self::Enabled
            } else {
                Self::Other(registered)
            }
        } else if registered.program.is_file() {
            Self::Other(registered)
        } else {
            Self::Missing(registered)
        }
    }
}

/// Whether `a` and `b` name the same program: equal paths, or paths that
/// resolve to the same file.
fn is_same_program(a: &Path, b: &Path) -> bool {
    a == b
        || matches!(
            (std::fs::canonicalize(a), std::fs::canonicalize(b)),
            (Ok(a), Ok(b)) if a == b
        )
}

#[cfg(test)]
#[path = "tests/autostart_tests.rs"]
mod tests;
