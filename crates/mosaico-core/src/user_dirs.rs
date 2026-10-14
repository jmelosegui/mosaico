//! Where the platform keeps each user's files.

use std::path::PathBuf;

/// Where the platform keeps the current user's base directories for
/// configuration and for local application state.
///
/// An [`Instance`](crate::Instance) keeps its files in a `mosaico` folder
/// inside each and applies the `MOSAICO_*` overrides, so a platform only
/// reports where the user's directories come from and what they hold.
///
/// # Contract for platform implementations
///
/// - `config_dir` is where the user keeps configuration they edit by
///   hand. mosaico keeps its config files and its logs there.
/// - `data_dir` is where applications keep state that is not
///   configuration. It must stay on this machine rather than follow the
///   user to others, and survive the daemon restarting. mosaico keeps
///   its PID file and its record of hidden windows there.
/// - On a platform mosaico already supports, keep the locations it has
///   always used, or existing installs lose their config. A new platform
///   follows its own convention, including the settings the user has for
///   it.
/// - Name the setting each directory comes from as the user knows it,
///   such as an environment variable, and report its value as found.
///   Never fall back to the working directory or a temporary one.
/// - Every process of the user with the same environment gets the same
///   answer, so the CLI and the daemon it starts agree.
/// - Do not create the directories; mosaico creates what it writes to.
/// - Do not read the `MOSAICO_*` variables; the instance applies them.
pub trait UserDirs {
    /// Where the base directory for the user's configuration comes from.
    fn config_dir(&self) -> UserDir;

    /// Where the base directory for application state kept on this
    /// machine comes from.
    fn data_dir(&self) -> UserDir;
}

/// One of the user's base directories, as the platform keeps it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserDir {
    /// What the user knows the setting by, such as the name of an
    /// environment variable, so a message can point at it.
    pub setting: &'static str,
    /// What the setting holds, or `None` when it is not set.
    pub value: Option<PathBuf>,
    /// The folder inside `value` that is the base directory, when the
    /// setting names a folder above it, such as the home folder.
    pub subfolder: Option<&'static str>,
}
