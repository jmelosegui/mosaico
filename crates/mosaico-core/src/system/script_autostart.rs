//! Starting the user's scripts when they log in, each on its own.

use std::path::PathBuf;

use crate::Result;

use super::LaunchCommand;

impl LaunchCommand {
    /// What runs the script called `name` at login: `exe script run
    /// <name> --background`, which starts the script detached, without a
    /// console, and returns.
    pub fn script(exe: impl Into<PathBuf>, name: &str) -> Self {
        Self {
            program: exe.into(),
            args: vec![
                "script".into(),
                "run".into(),
                name.into(),
                "--background".into(),
            ],
        }
    }
}

/// Starting scripts when the user logs in, apart from mosaico itself.
///
/// # Contract for platform implementations
///
/// - Each script has a registration of its own, named after the script,
///   per user and without elevation. It starts when the user's session
///   does, like mosaico's own ([`Autostart`](super::Autostart)), and
///   never replaces or removes mosaico's registration or another
///   script's.
/// - `enable` registers exactly the command it is given for `name`,
///   replacing that script's registration if there was one. A command the
///   platform cannot store as given is refused with
///   [`Error::Unsupported`](crate::Error::Unsupported).
/// - `is_enabled` is true while `name` has a registration, whether or not
///   the user turned it off in the system's own settings.
/// - `disable` removes the registration of `name`, and succeeds when
///   there was none.
/// - A failure in the system is an
///   [`Error::Platform`](crate::Error::Platform). A platform without a way
///   to start programs at login returns
///   [`Error::Unsupported`](crate::Error::Unsupported) from `enable`,
///   false from `is_enabled` and succeeds in `disable`.
pub trait ScriptAutostart {
    /// Whether the script called `name` starts at login.
    fn is_enabled(&self, name: &str) -> Result<bool>;

    /// Registers `command` to start the script called `name` at login.
    fn enable(&self, name: &str, command: &LaunchCommand) -> Result<()>;

    /// Stops the script called `name` starting at login.
    fn disable(&self, name: &str) -> Result<()>;
}
