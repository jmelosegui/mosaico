//! The user's folders, as Windows names them in the environment of every
//! logon session.
//!
//! mosaico keeps its configuration in `.config` in the profile folder
//! (`%USERPROFILE%\.config\mosaico`), like tools that share one config
//! layout across systems, rather than in the roaming application data
//! folder. Its state goes in the local application data folder
//! (`%LOCALAPPDATA%\mosaico`), which stays on this machine.

use std::ffi::OsString;
use std::path::PathBuf;

use mosaico_core::user_dirs::{UserDir, UserDirs};

/// The current user's folders, read from the environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsUserDirs {
    config: UserDir,
    data: UserDir,
}

impl WindowsUserDirs {
    /// The folders of the user this process runs as.
    pub fn current() -> Self {
        Self::from_env(|key| std::env::var_os(key))
    }

    /// The folders named by the variables `var` looks up, so tests can
    /// supply an environment of their own.
    fn from_env(var: impl Fn(&str) -> Option<OsString>) -> Self {
        let read = |setting: &'static str| UserDir {
            setting,
            value: var(setting).map(PathBuf::from),
            subfolder: None,
        };
        // HOME stands in only for a USERPROFILE that is not set at all,
        // as before.
        let mut profile = read("USERPROFILE");
        if profile.value.is_none() {
            let home = read("HOME");
            if home.value.is_some() {
                profile = home;
            }
        }
        Self {
            config: UserDir {
                subfolder: Some(".config"),
                ..profile
            },
            data: read("LOCALAPPDATA"),
        }
    }
}

impl UserDirs for WindowsUserDirs {
    /// `.config` in the profile folder: `USERPROFILE`, or `HOME` when
    /// `USERPROFILE` is not set.
    fn config_dir(&self) -> UserDir {
        self.config.clone()
    }

    /// The local application data folder, `LOCALAPPDATA`.
    fn data_dir(&self) -> UserDir {
        self.data.clone()
    }
}

#[cfg(test)]
#[path = "user_dirs_tests.rs"]
mod tests;
